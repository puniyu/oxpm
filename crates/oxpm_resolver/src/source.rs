use std::path::PathBuf;
use smol_str::SmolStr;

use oxpm_package_json::PackageJson;

use crate::cache::PackageCache;
use crate::error::Error;
use crate::types::{DepNode, DepType};
use crate::Result;
use crate::tarball::{download_and_extract, extract_tarball};

/// 解析 registry 来源
pub async fn resolve_registry(
    name: &SmolStr,
    range: &str,
    source: oxpm_common::RegistrySource,
    dep_type: DepType,
    registry: &oxpm_registry::Registry,
    cache: &PackageCache,
) -> Result<DepNode> {
    let pkg_info = registry
        .package(name.as_str())
        .await
        .map_err(|_| Error::PackageNotFound(name.clone()))?;

    let (version, package_version) = crate::version::select_version(name, range, &pkg_info)?;

    let source_type = oxpm_common::SourceType::Registry(source.clone());

    // 检查缓存
    if let Some(cached) = cache.get(name, &version, &source_type) {
        return Ok(cached);
    }

    let extracted = download_and_extract(&package_version.dist.tarball).await?;
    let pkg_json_path = extracted.join("package.json");
    let _pkg = PackageJson::load_from_path(&pkg_json_path)?;

    let integrity = package_version.dist.integrity.clone();

    let mut node = DepNode::new(name.clone(), version.clone(), dep_type)
        .with_source(source_type.clone())
        .with_range(SmolStr::new(range));
    if let Some(integrity) = integrity {
        node = node.with_integrity(integrity);
    }

    // 插入缓存
    cache.insert(name.clone(), version, source_type, node.clone());

    Ok(node)
}

/// 解析 file 来源
pub async fn resolve_file(
    name: &SmolStr,
    range: &str,
    source: oxpm_common::FileSource,
    dep_type: DepType,
    cache: &PackageCache,
) -> Result<DepNode> {
    let package_json_path = PathBuf::from(source.path()).join("package.json");
    let pkg = PackageJson::load_from_path(&package_json_path)?;
    let version = pkg.version.clone().ok_or_else(|| Error::VersionNotFound {
        name: name.clone(),
        range: SmolStr::new(range),
    })?;

    let source_type = oxpm_common::SourceType::File(source.clone());

    // 检查缓存
    if let Some(cached) = cache.get(name, &version, &source_type) {
        return Ok(cached);
    }

    let node = DepNode::new(name.clone(), version.clone(), dep_type)
        .with_source(source_type.clone())
        .with_range(SmolStr::new(range));

    // 插入缓存
    cache.insert(name.clone(), version, source_type, node.clone());

    Ok(node)
}

/// 解析 link 来源
pub async fn resolve_link(
    name: &SmolStr,
    range: &str,
    source: oxpm_common::LinkSource,
    dep_type: DepType,
    cache: &PackageCache,
) -> Result<DepNode> {
    let link_target = resolve_link_target(PathBuf::from(source.path()))?;
    let package_json_path = link_target.join("package.json");
    let pkg = PackageJson::load_from_path(&package_json_path)?;
    let version = pkg.version.clone().ok_or_else(|| Error::VersionNotFound {
        name: name.clone(),
        range: SmolStr::new(range),
    })?;

    let source_type = oxpm_common::SourceType::Link(source.clone());

    // 检查缓存
    if let Some(cached) = cache.get(name, &version, &source_type) {
        return Ok(cached);
    }

    let node = DepNode::new(name.clone(), version.clone(), dep_type)
        .with_source(source_type.clone())
        .with_range(SmolStr::new(range));

    // 插入缓存
    cache.insert(name.clone(), version, source_type, node.clone());

    Ok(node)
}

/// 解析 tarball 来源
pub async fn resolve_tarball(
    name: &SmolStr,
    range: &str,
    source: oxpm_common::TarballSource,
    dep_type: DepType,
    cache: &PackageCache,
) -> Result<DepNode> {
    let extracted = extract_tarball(source.path()).await?;
    let package_json_path = extracted.join("package.json");
    let pkg = PackageJson::load_from_path(&package_json_path)?;
    let version = pkg.version.clone().ok_or_else(|| Error::VersionNotFound {
        name: name.clone(),
        range: SmolStr::new(range),
    })?;

    let source_type = oxpm_common::SourceType::Tarball(source.clone());

    // 检查缓存
    if let Some(cached) = cache.get(name, &version, &source_type) {
        return Ok(cached);
    }

    let node = DepNode::new(name.clone(), version.clone(), dep_type)
        .with_source(source_type.clone())
        .with_range(SmolStr::new(range));

    // 插入缓存
    cache.insert(name.clone(), version, source_type, node.clone());

    Ok(node)
}

fn resolve_link_target(mut path: PathBuf) -> Result<PathBuf> {
    let mut visited = std::collections::HashSet::new();
    loop {
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            if !visited.insert(path.clone()) {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "symlink cycle detected",
                )));
            }
            let target = std::fs::read_link(&path)?;
            path = if target.is_absolute() {
                target
            } else {
                path.parent()
                    .map(|p| p.join(&target))
                    .unwrap_or(target)
            };
        } else {
            return Ok(path);
        }
    }
}