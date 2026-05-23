use oxpm_semver::Version;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepType {
    #[default]
    Production,
    Development,
    Peer,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepRoot {
    pub name: SmolStr,
    pub version: Version,
    pub path: PathBuf,
}

impl DepRoot {
    pub fn new(name: SmolStr, version: Version, path: PathBuf) -> Self {
        Self { name, version, path }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepNode {
    pub name: SmolStr,
    pub version: Version,
    pub source: SmolStr,
    pub range: SmolStr,
    pub dep_type: DepType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<SmolStr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<DepNode>,
}

impl DepNode {
    pub fn new(name: SmolStr, version: Version, dep_type: DepType, source: SmolStr) -> Self {
        Self {
            name,
            version,
            source,
            range: SmolStr::new(""),
            dep_type,
            integrity: None,
            children: Vec::new(),
        }
    }

    pub fn with_source(mut self, source: SmolStr) -> Self {
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
    pub root: DepRoot,
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

    pub fn flatten(&self) -> Vec<&DepNode> {
        self.nodes.iter().collect()
    }

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
    fn dep_type_default() {
        assert_eq!(DepType::default(), DepType::Production);
    }

    #[test]
    fn dep_node_new() {
        let node = DepNode::new(
            SmolStr::new("local-pkg"),
            Version::new(1, 0, 0),
            DepType::Production,
            SmolStr::new("file:../packages/local"),
        );
        assert_eq!(node.name.as_str(), "local-pkg");
        assert_eq!(node.version.to_string(), "1.0.0");
        assert_eq!(node.source.as_str(), "file:../packages/local");
        assert!(node.integrity.is_none());
        assert!(node.children.is_empty());
    }

    #[test]
    fn dependency_tree_new() {
        let root = DepRoot::new(SmolStr::new("my-app"), Version::new(1, 0, 0), PathBuf::from("."));
        let tree = DependencyTree::new(root);
        assert!(tree.nodes.is_empty());
        assert!(tree.is_empty());
    }

    #[test]
    fn dep_node_serialize() {
        let node = DepNode::new(
            SmolStr::new("lodash"),
            Version::new(4, 17, 21),
            DepType::Production,
            SmolStr::new("registry+https://registry.npmjs.org"),
        )
        .with_integrity(SmolStr::new("sha512-abc123"));

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("lodash"));
        assert!(json.contains("4.17.21"));
        assert!(json.contains("sha512-abc123"));
    }

    #[test]
    fn dep_node_with_source() {
        let node = DepNode::new(
            SmolStr::new("pkg"),
            Version::new(1, 0, 0),
            DepType::Production,
            SmolStr::new(""),
        )
        .with_source(SmolStr::new("git+https://github.com/user/repo#main"));

        assert_eq!(node.source.as_str(), "git+https://github.com/user/repo#main");
    }
}