use std::collections::HashMap;
use std::sync::RwLock;

use indexmap::IndexMap;
use oxpm_common::SourceType;
use oxpm_lockfile::{Lockfile as oxpmLockfile, Package as oxpmPackage};
use rayon::iter::{ParallelBridge, ParallelIterator};
use smol_str::SmolStr;

use crate::LockFileError;

use super::package;

pub(crate) struct Cache {
    /// 按 package key (name@version) 缓存转换结果，避免重复转换同名同版本包
    package_cache: RwLock<HashMap<SmolStr, oxpmPackage>>,
    /// 缓存 resolved URL -> SourceType 的解析结果
    source_cache: RwLock<HashMap<SmolStr, SourceType>>,
    /// 缓存 path -> package name 的解析结果
    name_cache: RwLock<HashMap<SmolStr, SmolStr>>,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            package_cache: RwLock::new(HashMap::new()),
            source_cache: RwLock::new(HashMap::new()),
            name_cache: RwLock::new(HashMap::new()),
        }
    }
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn packages_to_lockfile(
        &self,
        packages: Option<&IndexMap<SmolStr, package::Package>>,
    ) -> Result<oxpmLockfile, LockFileError> {
        let results: Vec<Result<(SmolStr, oxpmPackage), LockFileError>> = match packages {
            Some(packages) => packages
                .iter()
                .par_bridge()
                .filter(|(path, _)| !path.is_empty())
                .map(|(path, package)| {
                    let is_dev = package.dev.unwrap_or(false) || package.dev_optional.unwrap_or(false);
                    let is_optional = package.optional.unwrap_or(false);
                    let pkg = self.convert_package(path, package, is_dev, is_optional)?;
                    Ok((path.clone(), pkg))
                })
                .collect(),
            None => Vec::new(),
        };

        let mut output = oxpmLockfile::new();
        for result in results {
            let (_path, pkg) = result?;
            output.packages.push(pkg);
        }
        Ok(output)
    }

    fn convert_package(
        &self,
        path: &SmolStr,
        package: &package::Package,
        is_dev: bool,
        is_optional: bool,
    ) -> Result<oxpmPackage, LockFileError> {
        let name = package.name.clone().map(Ok).unwrap_or_else(|| self.package_name_from_path(path))?;
        let version = package.version.as_ref().ok_or_else(|| LockFileError::MissingPackageVersion {
            path: path.clone(),
            name: name.clone(),
        })?;

        let cache_key = format!("{}@{}", name, version).into();
        let cached_pkg = self.package_cache.read().unwrap().get(&cache_key).cloned();

        let source = self
            .resolve_source(package.resolved.as_ref())
            .unwrap_or_else(|| SourceType::parse("registry+https://registry.npmjs.org").unwrap());
        let has_bin = package.bin.is_some();

        let deps = package.dependencies.clone().map(|deps| dependency_specs(&deps));

        if let Some(mut pkg) = cached_pkg {
            pkg.dependencies = if is_dev || is_optional { None } else { deps.clone() };
            pkg.dev_dependencies = if is_dev { deps.clone() } else { None };
            pkg.optional_dependencies = if is_optional { package.optional_dependencies.clone().map(|d| vec_deps(&d)) } else { None };
            return Ok(pkg);
        }

        let pkg = oxpmPackage {
            name: name.clone(),
            version: version.clone(),
            source,
            integrity: package.integrity.clone(),
            dependencies: if is_dev { None } else { deps.clone() },
            dev_dependencies: if is_dev { deps } else { None },
            optional_dependencies: package.optional_dependencies.clone().map(|deps| vec_deps(&deps)),
            peer_dependencies: package.peer_dependencies.clone().map(|deps| vec_deps(&deps)),
            bin: if has_bin { Some(true) } else { None },
            engines: package.engines.clone(),
            os: package.os.clone(),
            cpu: package.cpu.clone(),
        };

        self.package_cache.write().unwrap().insert(cache_key, pkg.clone());
        Ok(pkg)
    }

    fn resolve_source(&self, resolved: Option<&SmolStr>) -> Option<SourceType> {
        let resolved = resolved?;
        if let Some(cached) = self.source_cache.read().unwrap().get(resolved) {
            return Some(cached.clone());
        }
        let source = if resolved.starts_with("http://") || resolved.starts_with("https://") {
            let url = url::Url::parse(resolved).ok()?;
            let origin = url.origin().ascii_serialization();
            SourceType::parse(&format!("registry+{origin}")).ok()
        } else {
            SourceType::parse(resolved).ok()
        }?;
        self.source_cache.write().unwrap().insert(resolved.clone(), source.clone());
        Some(source)
    }

    fn package_name_from_path(&self, path: &SmolStr) -> Result<SmolStr, LockFileError> {
        if let Some(cached) = self.name_cache.read().unwrap().get(path) {
            return Ok(cached.clone());
        }
        let name = Self::parse_name_from_path(path)?;
        self.name_cache.write().unwrap().insert(path.clone(), name.clone());
        Ok(name)
    }

    fn parse_name_from_path(path: &SmolStr) -> Result<SmolStr, LockFileError> {
        let mut parts = path.split(['/', '\\']);
        let mut name = None;

        while let Some(part) = parts.next() {
            if part != "node_modules" {
                continue;
            }

            let Some(first) = parts.next() else {
                return Err(LockFileError::InvalidPackagePath(path.clone()));
            };

            name = if first.starts_with('@') {
                let Some(second) = parts.next() else {
                    return Err(LockFileError::InvalidPackagePath(path.clone()));
                };
                Some(format!("{first}/{second}").into())
            } else {
                Some(first.into())
            };
        }

        name.ok_or_else(|| LockFileError::InvalidPackagePath(path.clone()))
    }
}

fn dependency_specs(deps: &IndexMap<SmolStr, SmolStr>) -> Vec<SmolStr> {
    deps.iter().map(|(n, v)| format!("{n}@{v}").into()).collect()
}

fn vec_deps(map: &IndexMap<SmolStr, SmolStr>) -> Vec<SmolStr> {
    map.iter().map(|(k, v)| format!("{k}@{v}").into()).collect()
}