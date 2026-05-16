mod version;
use std::path::Path;

pub use version::LockfileVersion;
mod package;
pub use package::*;

use oxpm_lockfile::{Lockfile as oxpmLockfile, Package as oxpmPackage};
use oxpm_semver::Version;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("failed to read lockfile")]
	Io(#[from] std::io::Error),
	#[error("unsupported lockfileVersion: {0}")]
	UnsupportedLockfileVersion(u64),
	#[error("missing package version for package {name} at {path}")]
	MissingPackageVersion { path: SmolStr, name: SmolStr },
	#[error("invalid npm package path: {0}")]
	InvalidPackagePath(SmolStr),
	#[error("failed to parse lockfile")]
	Parse(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<Version>,
	#[serde(rename = "lockfileVersion")]
	pub lockfile_version: version::LockfileVersion,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub requires: Option<bool>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub packages: IndexMap<SmolStr, package::Package>,
}

impl LockFile {
	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
		let content = std::fs::read_to_string(path)?;
		Self::load_from_str(&content)
	}
	pub fn load_from_str(content: &str) -> Result<Self> {
		let value: serde_json::Value = serde_json::from_str(content)?;
		let raw_version = value.get("lockfileVersion");
		if let Some(v) = raw_version {
			let v = v.as_u64().unwrap_or(0);
			LockfileVersion::try_from(v)?;
		}
		let lockfile = serde_json::from_value(value)?;
		Ok(lockfile)
	}

	pub fn to_string_pretty(&self) -> serde_json::Result<String> {
		serde_json::to_string_pretty(self)
	}

	pub fn to_oxpm_lockfile(&self) -> Result<oxpmLockfile> {
		self.try_into()
	}

	pub fn version(&self) -> version::LockfileVersion {
		self.lockfile_version
	}
}

impl TryFrom<&LockFile> for oxpmLockfile {
	type Error = Error;

	#[inline]
	fn try_from(value: &LockFile) -> Result<Self> {
		packages_to_lockfile(&value.packages)
	}
}

impl TryFrom<LockFile> for oxpmLockfile {
	type Error = Error;

	#[inline]
	fn try_from(value: LockFile) -> Result<Self> {
		(&value).try_into()
	}
}

fn packages_to_lockfile(packages: &IndexMap<SmolStr, package::Package>) -> Result<oxpmLockfile> {
	let mut output = oxpmLockfile::new();

	for (path, package) in packages {
		if path.is_empty() {
			continue;
		}

		output.packages.push(convert_package(path, package)?);
	}

	Ok(output)
}

fn convert_package(path: &SmolStr, package: &package::Package) -> Result<oxpmPackage> {
	let name = package.name.clone().map(Ok).unwrap_or_else(|| package_name_from_path(path))?;
	let version = package
		.version
		.as_ref()
		.ok_or_else(|| Error::MissingPackageVersion { path: path.clone(), name: name.clone() })?;
	let source = resolve_source(package.resolved.as_ref()).unwrap_or_else(|| "".into());
	let has_bin = package.bin.is_some();

	Ok(oxpmPackage {
		name: name.clone(),
		version: version.clone(),
		source,
		integrity: package.integrity.clone(),
		dependencies: dependency_specs(&package.dependencies),
		dev_dependencies: None,
		optional_dependencies: optional_vec_deps(&package.optional_dependencies),
		peer_dependencies: optional_vec_deps(&package.peer_dependencies),
		bin: if has_bin { Some(true) } else { None },
		engines: optional_map(&package.engines),
		os: optional_vec(&package.os),
		cpu: optional_vec(&package.cpu),
	})
}

fn package_name_from_path(path: &SmolStr) -> Result<SmolStr> {
	let mut parts = path.split(['/', '\\']);
	let mut name = None;

	while let Some(part) = parts.next() {
		if part != "node_modules" {
			continue;
		}

		let Some(first) = parts.next() else {
			return Err(Error::InvalidPackagePath(path.clone()));
		};

		name = if first.starts_with('@') {
			let Some(second) = parts.next() else {
				return Err(Error::InvalidPackagePath(path.clone()));
			};
			Some(format!("{first}/{second}").into())
		} else {
			Some(first.into())
		};
	}

	name.ok_or_else(|| Error::InvalidPackagePath(path.clone()))
}

fn resolve_source(resolved: Option<&SmolStr>) -> Option<SmolStr> {
	match resolved {
		Some(value) if value.starts_with("http://") || value.starts_with("https://") => {
			let url = url::Url::parse(value).ok()?;
			let origin = url.origin().ascii_serialization();
			Some(format!("registry+{origin}").into())
		}
		Some(value) => Some(value.clone()),
		None => None,
	}
}

fn dependency_specs(dependencies: &IndexMap<SmolStr, SmolStr>) -> Option<Vec<SmolStr>> {
	if dependencies.is_empty() {
		return None;
	}
	Some(dependencies.iter().map(|(name, version)| format!("{name}@{version}").into()).collect())
}

fn optional_map(map: &IndexMap<SmolStr, SmolStr>) -> Option<IndexMap<SmolStr, SmolStr>> {
	if map.is_empty() { None } else { Some(map.clone()) }
}

fn optional_vec(values: &[SmolStr]) -> Option<Vec<SmolStr>> {
	if values.is_empty() { None } else { Some(values.to_vec()) }
}

fn optional_vec_deps(map: &IndexMap<SmolStr, SmolStr>) -> Option<Vec<SmolStr>> {
	if map.is_empty() {
		None
	} else {
		Some(map.iter().map(|(k, v)| format!("{k}@{v}").into()).collect())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_v3_package_lock() {
		let json = r#"{
            "name": "demo",
            "version": "1.0.0",
            "lockfileVersion": 3,
            "requires": true,
            "packages": {
                "": {"name": "demo", "version": "1.0.0"},
                "node_modules/lodash": {"version": "4.17.21"}
            }
        }"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		assert_eq!(lockfile.version(), LockfileVersion::V3);
		assert_eq!(
			lockfile
				.packages
				.get("node_modules/lodash")
				.unwrap()
				.version
				.as_ref()
				.unwrap()
				.to_string(),
			"4.17.21"
		);
	}

	#[test]
	fn reject_v1_lockfile() {
		let err = LockFile::load_from_str(r#"{"lockfileVersion": 1}"#).unwrap_err();
		assert!(err.to_string().contains("unsupported"));
	}

	#[test]
	fn reject_v2_lockfile() {
		let err = LockFile::load_from_str(r#"{"lockfileVersion": 2}"#).unwrap_err();
		assert!(err.to_string().contains("unsupported"));
	}

	#[test]
	fn reject_unsupported_version() {
		let err = LockFile::load_from_str(r#"{"lockfileVersion": 4}"#).unwrap_err();
		assert!(err.to_string().contains("unsupported"));
	}

	#[test]
	fn lockfile_version_as_u32() {
		assert_eq!(LockfileVersion::V3.as_u32(), 3);
	}

	#[test]
	fn lockfile_version_try_from() {
		use super::Error;

		assert!(matches!(
			LockfileVersion::try_from(1u64),
			Err(Error::UnsupportedLockfileVersion(1))
		));
		assert!(matches!(
			LockfileVersion::try_from(2u64),
			Err(Error::UnsupportedLockfileVersion(2))
		));
		assert_eq!(LockfileVersion::try_from(3u64).unwrap(), LockfileVersion::V3);
		assert!(matches!(
			LockfileVersion::try_from(99u64),
			Err(Error::UnsupportedLockfileVersion(99))
		));
	}

	#[test]
	fn convert_v3_package_fields_to_oxpm_lockfile() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/@types/node": {
					"version": "20.11.0",
					"resolved": "git+https://example.com/types-node#abc",
					"dependencies": {"undici-types": "~5.26.4"},
					"bin": {"tsc": "bin/tsc"},
					"engines": {"node": ">=18"},
					"os": ["linux"],
					"cpu": ["x64"]
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
		let pkg = &oxpm_lockfile.packages[0];
		assert_eq!(pkg.name.as_str(), "@types/node");
		assert_eq!(pkg.source.as_str(), "git+https://example.com/types-node#abc");
		assert_eq!(pkg.dependencies.as_ref().unwrap()[0].as_str(), "undici-types@~5.26.4");
		assert!(pkg.bin.is_some());
		assert_eq!(pkg.engines.as_ref().unwrap().get("node").unwrap().as_str(), ">=18");
		assert_eq!(pkg.os.as_ref().unwrap()[0].as_str(), "linux");
		assert_eq!(pkg.cpu.as_ref().unwrap()[0].as_str(), "x64");
	}

	#[test]
	fn convert_missing_package_version_returns_error() {
		use super::Error;

		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/lodash": {}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		assert!(matches!(lockfile.to_oxpm_lockfile(), Err(Error::MissingPackageVersion { .. })));
	}

	#[test]
	fn convert_invalid_package_version_returns_error() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/lodash": {"version": "bad"}
			}
		}"#;

		let err = LockFile::load_from_str(json).unwrap_err();
		assert!(matches!(err, Error::Parse(_)));
	}

	#[test]
	fn convert_bin_path_uses_package_name() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/oxpm": {
					"version": "1.0.0",
					"bin": "bin/oxpm.js"
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
		assert!(oxpm_lockfile.packages[0].bin.is_some());
	}

	#[test]
	fn convert_nested_node_modules_path_infers_last_package_name() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/a/node_modules/b": {
					"version": "2.0.0"
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
		assert_eq!(oxpm_lockfile.packages[0].name.as_str(), "b");
	}

	#[test]
	fn convert_invalid_package_path_returns_error() {
		use super::Error;

		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"weird/path": {
					"version": "1.0.0"
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		assert!(matches!(lockfile.to_oxpm_lockfile(), Err(Error::InvalidPackagePath(_))));
	}

	#[test]
	fn convert_empty_collection_fields_to_none() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/lodash": {
					"version": "4.17.21",
					"dependencies": {},
					"engines": {},
					"os": [],
					"cpu": []
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
		let package = &oxpm_lockfile.packages[0];
		assert!(package.dependencies.is_none());
		assert!(package.engines.is_none());
		assert!(package.os.is_none());
		assert!(package.cpu.is_none());
	}

	#[test]
	fn convert_package_name_field_takes_priority_over_path() {
		let json = r#"{
			"lockfileVersion": 3,
			"packages": {
				"node_modules/foo": {
					"name": "bar",
					"version": "1.0.0"
				}
			}
		}"#;

		let lockfile = LockFile::load_from_str(json).unwrap();
		let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
		assert_eq!(oxpm_lockfile.packages[0].name.as_str(), "bar");
	}

	#[test]
	fn infer_scoped_package_name_from_path() {
		assert_eq!(
			package_name_from_path(&"node_modules/@types/node".into()).unwrap().as_str(),
			"@types/node"
		);
	}

	#[test]
	fn resolve_source_non_http_keeps_original() {
		let source = resolve_source(Some(&"git+https://example.com/repo#abc".into()));
		assert_eq!(source.unwrap().as_str(), "git+https://example.com/repo#abc");
	}

	#[test]
	fn resolve_source_http_extracts_registry_origin() {
		let source =
			resolve_source(Some(&"https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".into()));
		assert_eq!(source.unwrap().as_str(), "registry+https://registry.npmjs.org");
	}
}
