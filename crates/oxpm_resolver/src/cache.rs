use std::collections::HashMap;
use std::sync::RwLock;

use oxpm_common::SourceType;
use oxpm_semver::Version;
use smol_str::SmolStr;

use crate::types::DepNode;


#[derive(Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    name: SmolStr,
    version: Version,
    source: SourceType,
}

impl CacheKey {
    fn new(name: SmolStr, version: Version, source: SourceType) -> Self {
        Self { name, version, source }
    }
}


pub(crate) struct PackageCache {
    resolved: RwLock<HashMap<CacheKey, DepNode>>,
}

impl PackageCache {

    pub fn new() -> Self {
        Self {
            resolved: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &SmolStr, version: &Version, source: &SourceType) -> Option<DepNode> {
        let key = CacheKey::new(name.clone(), version.clone(), source.clone());
        self.resolved.read().ok()?.get(&key).cloned()
    }

    pub fn insert(&self, name: SmolStr, version: Version, source: SourceType, node: DepNode) {
        let key = CacheKey::new(name, version, source);
        if let Ok(mut guard) = self.resolved.write() {
            guard.insert(key, node);
        }
    }


    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut guard) = self.resolved.write() {
            guard.clear();
        }
    }
}

impl Default for PackageCache {
    fn default() -> Self {
        Self::new()
    }
}