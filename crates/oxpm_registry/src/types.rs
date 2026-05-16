use oxpm_package_json::{PackageBin, PackageRepository, Person};
use oxpm_semver::Version;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use url::Url;

/// 包的完整元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: SmolStr,
    #[serde(default, rename = "dist-tags")]
    pub dist_tags: IndexMap<SmolStr, SmolStr>,
    #[serde(default)]
    pub versions: IndexMap<SmolStr, PackageVersion>,
    #[serde(default)]
    pub time: IndexMap<SmolStr, SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<PackageRepository>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<Person>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<SmolStr>,
}

/// 单个版本的元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageVersion {
    pub name: SmolStr,
    pub version: Version,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<PackageRepository>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<IndexMap<SmolStr, SmolStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_dependencies_meta: Option<IndexMap<SmolStr, IndexMap<SmolStr, serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_dependencies: Option<IndexMap<SmolStr, SmolStr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<PackageBin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engines: Option<IndexMap<SmolStr, SmolStr>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub os: Vec<SmolStr>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cpu: Vec<SmolStr>,
    pub dist: Dist,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub maintainers: Vec<Person>,
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<SmolStr>,
    #[serde(rename = "_npmUser", skip_serializing_if = "Option::is_none")]
    pub npm_user: Option<Person>,
    #[serde(rename = "_hasShrinkwrap", skip_serializing_if = "Option::is_none")]
    pub has_shrinkwrap: Option<bool>,
}

/// tarball 下载与完整性校验信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dist {
    pub tarball: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shasum: Option<SmolStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<SmolStr>,
    #[serde(rename = "fileCount", skip_serializing_if = "Option::is_none")]
    pub file_count: Option<u64>,
    #[serde(rename = "unpackedSize", skip_serializing_if = "Option::is_none")]
    pub unpacked_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signatures: Vec<Signature>,
}

/// registry 签名。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub keyid: SmolStr,
    pub sig: SmolStr,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de<T: serde::de::DeserializeOwned>(json: &str) -> T {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn parse_dist_full() {
        let d: Dist = de(r#"{
            "tarball": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
            "shasum": "679591c564c3bffaae8454cf0b3df370c3d6911c",
            "integrity": "sha512-v2kDE/hTHE+abc123==",
            "fileCount": 1054,
            "unpackedSize": 1412415,
            "signatures": [{"keyid": "SHA256:key1", "sig": "abc123"}]
        }"#);
        assert_eq!(d.tarball.as_str(), "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz");
        assert_eq!(d.shasum.unwrap().as_str(), "679591c564c3bffaae8454cf0b3df370c3d6911c");
        assert_eq!(d.integrity.unwrap().as_str(), "sha512-v2kDE/hTHE+abc123==");
        assert_eq!(d.file_count, Some(1054));
        assert_eq!(d.unpacked_size, Some(1412415));
        assert_eq!(d.signatures.len(), 1);
        assert_eq!(d.signatures[0].keyid.as_str(), "SHA256:key1");
    }

    #[test]
    fn parse_dist_minimal() {
        let d: Dist = de(r#"{"tarball": "https://example.com/a.tgz"}"#);
        assert_eq!(d.tarball.as_str(), "https://example.com/a.tgz");
        assert!(d.shasum.is_none());
        assert!(d.integrity.is_none());
        assert!(d.file_count.is_none());
        assert!(d.unpacked_size.is_none());
        assert!(d.signatures.is_empty());
    }

    #[test]
    fn parse_package_version_minimal() {
        let v: PackageVersion = de(r#"{
            "name": "foo",
            "version": "1.0.0",
            "dist": {"tarball": "https://example.com/foo-1.0.0.tgz"}
        }"#);
        assert_eq!(v.name.as_str(), "foo");
        assert_eq!(v.version.to_string(), "1.0.0");
        assert!(v.description.is_none());
        assert!(v.dependencies.is_none());
        assert!(v.bin.is_none());
        assert!(v.maintainers.is_empty());
    }

    #[test]
    fn parse_package_version_full() {
        let v: PackageVersion = de(r#"{
            "name": "@scope/pkg",
            "version": "2.1.0",
            "description": "A package",
            "license": "MIT",
            "main": "./dist/index.js",
            "homepage": "https://example.com",
            "repository": {"type": "git", "url": "https://github.com/u/r.git"},
            "keywords": ["test"],
            "dependencies": {"lodash": "^4.17.21"},
            "devDependencies": {"jest": "^29.0.0"},
            "peerDependencies": {"react": ">=18"},
            "peerDependenciesMeta": {"react": {"optional": true}},
            "optionalDependencies": {"fsevents": "^2.3.0"},
            "bin": {"cli": "./cli.js"},
            "engines": {"node": ">=18"},
            "os": ["linux", "darwin"],
            "cpu": ["x64"],
            "dist": {
                "tarball": "https://example.com/pkg-2.1.0.tgz",
                "shasum": "abc123",
                "integrity": "sha512-xyz=="
            },
            "maintainers": [{"name": "user", "email": "u@e.com"}],
            "_id": "@scope/pkg@2.1.0",
            "_npmUser": {"name": "publisher"},
            "_hasShrinkwrap": false
        }"#);
        assert_eq!(v.name.as_str(), "@scope/pkg");
        assert_eq!(v.license.unwrap().as_str(), "MIT");
        assert_eq!(v.dependencies.unwrap().len(), 1);
        assert_eq!(v.dev_dependencies.unwrap().len(), 1);
        assert_eq!(v.peer_dependencies.unwrap().len(), 1);
        assert_eq!(v.peer_dependencies_meta.unwrap().len(), 1);
        assert_eq!(v.optional_dependencies.unwrap().len(), 1);
        assert!(v.bin.is_some());
        assert_eq!(v.engines.unwrap().get("node").unwrap().as_str(), ">=18");
        assert_eq!(v.os.len(), 2);
        assert_eq!(v.cpu.len(), 1);
        assert_eq!(v.maintainers.len(), 1);
        assert_eq!(v.id.unwrap().as_str(), "@scope/pkg@2.1.0");
        assert!(v.npm_user.is_some());
        assert_eq!(v.has_shrinkwrap, Some(false));
    }

    #[test]
    fn parse_package_minimal() {
        let p: Package = de(r#"{"name": "foo"}"#);
        assert_eq!(p.name.as_str(), "foo");
        assert!(p.dist_tags.is_empty());
        assert!(p.versions.is_empty());
        assert!(p.time.is_empty());
        assert!(p.description.is_none());
        assert!(p.readme.is_none());
        assert!(p.license.is_none());
        assert!(p.maintainers.is_empty());
        assert!(p.keywords.is_empty());
    }

    #[test]
    fn parse_package_with_versions() {
        let p: Package = de(r#"{
            "name": "foo",
            "dist-tags": {"latest": "1.0.0"},
            "versions": {
                "1.0.0": {
                    "name": "foo",
                    "version": "1.0.0",
                    "dist": {"tarball": "https://example.com/foo-1.0.0.tgz"}
                }
            },
            "time": {
                "created": "2024-01-01T00:00:00.000Z",
                "1.0.0": "2024-01-01T00:00:00.000Z"
            },
            "description": "A test package",
            "license": "MIT",
            "homepage": "https://example.com",
            "repository": "github:user/repo",
            "maintainers": [{"name": "user"}],
            "keywords": ["test", "example"]
        }"#);
        assert_eq!(p.dist_tags.get("latest").unwrap().as_str(), "1.0.0");
        assert_eq!(p.versions.len(), 1);
        assert_eq!(p.time.len(), 2);
        assert_eq!(p.description.unwrap().as_str(), "A test package");
        assert_eq!(p.license.unwrap().as_str(), "MIT");
        assert_eq!(p.maintainers.len(), 1);
        assert_eq!(p.keywords.len(), 2);

        let v = p.versions.get("1.0.0").unwrap();
        assert_eq!(v.name.as_str(), "foo");
    }

    #[test]
    fn roundtrip_package_version() {
        let json = r#"{
            "name": "bar",
            "version": "0.1.0",
            "dist": {"tarball": "https://example.com/bar-0.1.0.tgz"}
        }"#;
        let v: PackageVersion = de(json);
        let serialized = serde_json::to_string(&v).unwrap();
        let v2: PackageVersion = de(&serialized);
        assert_eq!(v2.name.as_str(), "bar");
        assert_eq!(v2.version.to_string(), "0.1.0");
    }

    #[test]
    fn roundtrip_package() {
        let json = r#"{
            "name": "baz",
            "dist-tags": {"latest": "1.0.0"},
            "versions": {
                "1.0.0": {
                    "name": "baz",
                    "version": "1.0.0",
                    "dist": {"tarball": "https://example.com/baz-1.0.0.tgz"}
                }
            }
        }"#;
        let p: Package = de(json);
        let serialized = serde_json::to_string(&p).unwrap();
        let p2: Package = de(&serialized);
        assert_eq!(p2.name.as_str(), "baz");
        assert_eq!(p2.dist_tags.get("latest").unwrap().as_str(), "1.0.0");
        assert_eq!(p2.versions.len(), 1);
    }

    #[test]
    fn signature_parse() {
        let s: Signature = de(r#"{"keyid": "SHA256:abc", "sig": "meow123"}"#);
        assert_eq!(s.keyid.as_str(), "SHA256:abc");
        assert_eq!(s.sig.as_str(), "meow123");
    }
}
