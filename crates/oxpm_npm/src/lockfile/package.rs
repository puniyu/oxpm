use std::hash::Hash;
use indexmap::IndexMap;
use serde::Deserialize;
use smol_str::SmolStr;
use oxpm_package_json::PackageBin;
use oxpm_semver::Version;

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

fn empty_vec_as_none<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
	D: serde::Deserializer<'de>,
	T: Deserialize<'de>,
{
	Ok(Option::<Vec<T>>::deserialize(deserializer)?.filter(|v| !v.is_empty()))
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
pub struct Package {
	pub name: Option<SmolStr>,
	pub version: Option<Version>,
	pub resolved: Option<SmolStr>,
	#[serde(default, deserialize_with = "empty_str_as_none")]
	pub integrity: Option<SmolStr>,
	#[serde(default, deserialize_with = "empty_str_as_none")]
	pub license: Option<SmolStr>,
	pub link: Option<bool>,
	pub dev: Option<bool>,
	pub optional: Option<bool>,
	pub dev_optional: Option<bool>,
	pub in_bundle: Option<bool>,
	pub has_install_script: Option<bool>,
	pub bin: Option<PackageBin>,
	#[serde(default, deserialize_with = "empty_map_as_none")]
	pub dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	#[serde(default, deserialize_with = "empty_map_as_none")]
	pub optional_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	#[serde(default, deserialize_with = "empty_map_as_none")]
	pub peer_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
	#[serde(default)]
	pub peer_dependencies_meta: Option<IndexMap<SmolStr, PeerDependencyMeta>>,
	#[serde(default, deserialize_with = "empty_map_as_none")]
	pub engines: Option<IndexMap<SmolStr, SmolStr>>,
	#[serde(default, deserialize_with = "empty_vec_as_none")]
	pub cpu: Option<Vec<SmolStr>>,
	#[serde(default, deserialize_with = "empty_vec_as_none")]
	pub os: Option<Vec<SmolStr>>,
	#[serde(default, deserialize_with = "empty_vec_as_none")]
	pub bundled_dependencies: Option<Vec<SmolStr>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerDependencyMeta {
	pub optional: Option<bool>,
}