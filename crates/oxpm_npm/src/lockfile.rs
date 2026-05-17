mod cache;
mod package;

pub use package::*;

mod version;

pub use version::LockfileVersion;

use std::hash::Hash;
use indexmap::IndexMap;
use oxpm_lockfile::Lockfile as oxpmLockfile;
use serde::Deserialize;
use smol_str::SmolStr;
use std::path::Path;

use crate::LockFileError;

fn empty_str_as_none<'de, D>(deserializer: D) -> Result<Option<SmolStr>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	match Option::<String>::deserialize(deserializer)? {
		None => Ok(None),
		Some(s) if s.is_empty() => Ok(None),
		Some(s) => Ok(Some(s.into())),
	}
}

fn empty_version_as_none<'de, D>(deserializer: D) -> Result<Option<oxpm_semver::Version>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	Ok(Option::<oxpm_semver::Version>::deserialize(deserializer)?.filter(|v| {
		let s = v.to_string();
		!s.is_empty()
	}))
}

fn empty_map_as_none<'de, D, K, V>(deserializer: D) -> Result<Option<IndexMap<K, V>>, D::Error>
where
	D: serde::Deserializer<'de>,
	K: Deserialize<'de> + Eq + Hash,
	V: Deserialize<'de>,
{
	Ok(Option::<IndexMap<K, V>>::deserialize(deserializer)?.filter(|m| !m.is_empty()))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockFile {
	#[serde(default, deserialize_with = "empty_str_as_none")]
	pub name: Option<SmolStr>,
	#[serde(default, deserialize_with = "empty_version_as_none")]
	pub version: Option<oxpm_semver::Version>,
	#[serde(rename = "lockfileVersion")]
	pub lockfile_version: LockfileVersion,
	pub requires: Option<bool>,
	#[serde(default, deserialize_with = "empty_map_as_none")]
	pub packages: Option<IndexMap<SmolStr, package::Package>>,
}

impl LockFile {
	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, LockFileError> {
		let content = std::fs::read_to_string(path)?;
		Self::load_from_str(&content)
	}

	pub fn load_from_str(content: &str) -> Result<Self, LockFileError> {
		let value: serde_json::Value = serde_json::from_str(content)?;
		let raw_version = value.get("lockfileVersion");
		if let Some(v) = raw_version {
			let v = v.as_u64().unwrap_or(0);
			LockfileVersion::try_from(v)?;
		}
		let lockfile = serde_json::from_value(value)?;
		Ok(lockfile)
	}

	pub fn to_oxpm_lockfile(&self) -> Result<oxpmLockfile, LockFileError> {
		self.try_into()
	}

	pub fn version(&self) -> version::LockfileVersion {
		self.lockfile_version
	}
}

impl TryFrom<&LockFile> for oxpmLockfile {
	type Error = LockFileError;

	#[inline]
	fn try_from(value: &LockFile) -> Result<Self, LockFileError> {
		packages_to_lockfile(value.packages.as_ref())
	}
}

impl TryFrom<LockFile> for oxpmLockfile {
	type Error = LockFileError;

	#[inline]
	fn try_from(value: LockFile) -> Result<Self, LockFileError> {
		packages_to_lockfile(value.packages.as_ref())
	}
}

fn packages_to_lockfile(
	packages: Option<&IndexMap<SmolStr, package::Package>>,
) -> Result<oxpmLockfile, LockFileError> {
	cache::Cache::new().packages_to_lockfile(packages)
}