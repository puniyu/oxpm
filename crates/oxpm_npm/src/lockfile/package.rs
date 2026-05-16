use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use oxpm_package_json::PackageBin;
use oxpm_semver::Version;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub version: Option<Version>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub resolved: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub integrity: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub license: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub link: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dev_optional: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub in_bundle: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub has_install_script: Option<bool>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bin: Option<PackageBin>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub dependencies: IndexMap<SmolStr, SmolStr>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub optional_dependencies: IndexMap<SmolStr, SmolStr>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub peer_dependencies: IndexMap<SmolStr, SmolStr>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub peer_dependencies_meta: IndexMap<SmolStr, PeerDependencyMeta>,
	#[serde(default, skip_serializing_if = "IndexMap::is_empty")]
	pub engines: IndexMap<SmolStr, SmolStr>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub cpu: Vec<SmolStr>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub os: Vec<SmolStr>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub bundled_dependencies: Vec<SmolStr>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerDependencyMeta {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub optional: Option<bool>,
}
