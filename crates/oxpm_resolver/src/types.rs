use std::path::PathBuf;

use oxpm_common::SourceType;
use oxpm_semver::{Version, VersionRangeKind};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// 依赖类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepType {
	/// dependencies
	#[default]
	Production,
	/// devDependencies
	Development,
	/// peerDependencies
	Peer,
	/// optionalDependencies
	Optional,
}

impl DepType {
	pub fn is_prod(&self) -> bool {
		matches!(self, Self::Production)
	}
	pub fn is_dev(&self) -> bool {
		matches!(self, Self::Development)
	}
	pub fn is_peer(&self) -> bool {
		matches!(self, Self::Peer)
	}
	pub fn is_optional(&self) -> bool {
		matches!(self, Self::Optional)
	}
}

#[derive(Debug, Clone)]
pub(crate) struct DepTask {
	/// 包名
	pub name: SmolStr,
	/// 版本范围
	pub range: VersionRangeKind,
	/// 依赖类型
	pub dep_type: DepType,
}

impl DepTask {
	pub fn new(name: SmolStr, range: VersionRangeKind, dep_type: DepType) -> Self {
		Self { name, range, dep_type }
	}
}

/// 根节点（表示待解析的根包，来自本地 package.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepRoot {
	/// 包名
	pub name: SmolStr,
	/// 版本
	pub version: Version,
	/// 本地路径
	pub path: PathBuf,
}

impl DepRoot {
	pub fn new(name: SmolStr, version: Version, path: PathBuf) -> Self {
		Self { name, version, path }
	}
}

/// 依赖节点（表示解析出的依赖）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepNode {
	/// 包名，例如 `lodash` 或 `@scope/pkg`
	pub name: SmolStr,
	/// 解析后的精确版本号
	pub version: Version,
	/// 包来源
	pub source: SourceType,
	/// package.json 中的原始范围字符串
	pub range: SmolStr,
	/// 依赖类型
	pub dep_type: DepType,
	/// SRI 完整性哈希
	#[serde(skip_serializing_if = "Option::is_none")]
	pub integrity: Option<SmolStr>,
	/// 直接依赖
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub children: Vec<DepNode>,
}

impl DepNode {
	pub fn new(name: SmolStr, version: Version, dep_type: DepType) -> Self {
		Self { name, version, source: SourceType::default(), range: SmolStr::new(""), dep_type, integrity: None, children: Vec::new() }
	}

	pub fn with_source(mut self, source: SourceType) -> Self {
		self.source = source;
		self
	}

	pub fn with_range(mut self, range: SmolStr) -> Self {
		self.range = range;
		self
	}

	pub fn with_integrity(mut self, integrity: SmolStr) -> Self {
		self.integrity = Some(integrity);
		self
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyTree {
	/// 根节点
	pub root: DepRoot,
	/// 所有依赖节点
	#[serde(default)]
	pub nodes: Vec<DepNode>,
}

impl DependencyTree {
	pub fn new(root: DepRoot) -> Self {
		Self { root, nodes: Vec::new() }
	}

	pub fn root(&self) -> &DepRoot {
		&self.root
	}

	pub fn root_mut(&mut self) -> &mut DepRoot {
		&mut self.root
	}

	/// 扁平化输出所有节点
	pub fn flatten(&self) -> Vec<&DepNode> {
		self.nodes.iter().collect()
	}

	/// 统计依赖节点数
	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_dep_type_default() {
		assert_eq!(DepType::default(), DepType::Production);
	}

	#[test]
	fn test_dep_type_is_prod() {
		assert!(DepType::Production.is_prod());
		assert!(!DepType::Production.is_dev());
		assert!(!DepType::Production.is_peer());
		assert!(!DepType::Production.is_optional());
	}

	#[test]
	fn test_dep_type_is_dev() {
		assert!(DepType::Development.is_dev());
		assert!(!DepType::Development.is_prod());
	}

	#[test]
	fn test_dep_type_is_peer() {
		assert!(DepType::Peer.is_peer());
	}

	#[test]
	fn test_dep_type_is_optional() {
		assert!(DepType::Optional.is_optional());
	}

	#[test]
	fn test_dep_root_new() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		assert_eq!(root.name.as_str(), "my-app");
		assert_eq!(root.version.to_string(), "1.0.0");
		assert_eq!(root.path, PathBuf::from("."));
	}

	#[test]
	fn test_dep_node_new() {
		let node = DepNode::new(SmolStr::new("lodash"), Version::new(4, 17, 21), DepType::Production);
		assert_eq!(node.name.as_str(), "lodash");
		assert_eq!(node.version.to_string(), "4.17.21");
		assert!(node.integrity.is_none());
		assert!(node.children.is_empty());
	}

	#[test]
	fn test_dep_node_with_integrity() {
		let node = DepNode::new(SmolStr::new("lodash"), Version::new(4, 17, 21), DepType::Production)
			.with_integrity(SmolStr::new("sha512-abc123"));

		assert_eq!(node.integrity.as_deref(), Some("sha512-abc123"));
	}

	#[test]
	fn test_dependency_tree_new() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		let tree = DependencyTree::new(root);

		assert_eq!(tree.root.name.as_str(), "my-app");
		assert_eq!(tree.root.version.to_string(), "1.0.0");
		assert!(tree.nodes.is_empty());
		assert!(tree.is_empty());
		assert_eq!(tree.len(), 0);
	}

	#[test]
	fn test_dependency_tree_add_nodes() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		let mut tree = DependencyTree::new(root);

		let node = DepNode::new(SmolStr::new("lodash"), Version::new(4, 17, 21), DepType::Production);
		tree.nodes.push(node);

		assert!(!tree.is_empty());
		assert_eq!(tree.len(), 1);
	}

	#[test]
	fn test_dependency_tree_root_mut() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		let mut tree = DependencyTree::new(root);

		let tree_root = tree.root();
		assert_eq!(tree_root.name.as_str(), "my-app");

		let tree_mut_root = tree.root_mut();
		assert_eq!(tree_mut_root.name.as_str(), "my-app");
	}

	#[test]
	fn test_dependency_tree_flatten() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		let mut tree = DependencyTree::new(root);

		for i in 0..5 {
			let node = DepNode::new(SmolStr::new(format!("dep-{}", i)), Version::new(1, 0, i), DepType::Production);
			tree.nodes.push(node);
		}

		let flat = tree.flatten();
		assert_eq!(flat.len(), 5);
	}

	#[test]
	fn test_dependency_tree_serialize() {
		let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
		let tree = DependencyTree::new(root);

		let json = serde_json::to_string(&tree).unwrap();
		assert!(json.contains("my-app"));
		assert!(json.contains("1.0.0"));
	}

	#[test]
	fn test_dep_node_serialize() {
		let node = DepNode::new(SmolStr::new("lodash"), Version::new(4, 17, 21), DepType::Production)
			.with_integrity(SmolStr::new("sha512-abc123"));

		let json = serde_json::to_string(&node).unwrap();
		assert!(json.contains("lodash"));
		assert!(json.contains("4.17.21"));
		assert!(json.contains("sha512-abc123"));
	}

	#[test]
	fn test_dependency_tree_deserialize() {
		let json = r#"{"root":{"name":"my-app","version":"1.0.0","path":"."},"nodes":[]}"#;
		let tree: DependencyTree = serde_json::from_str(json).unwrap();

		assert_eq!(tree.root.name.as_str(), "my-app");
		assert_eq!(tree.root.version.to_string(), "1.0.0");
	}
}