use oxpm_semver::Version;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;


fn skip_bin(v: &Option<bool>) -> bool {
    v.is_none_or(|b| !b)
}

fn skip_empty_vec<T>(v: &Option<Vec<T>>) -> bool {
    v.as_ref().is_none_or(Vec::is_empty)
}

fn skip_empty_indexmap<K, V>(m: &Option<IndexMap<K, V>>) -> bool {
    m.as_ref().is_none_or(IndexMap::is_empty)
}

/// 锁文件中的单个包条目，记录该包的精确版本及来源信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
	/// 包名，例如 `lodash` 或 `@scope/pkg`。
	pub name: SmolStr,
	/// 锁定的精确版本号。
	pub version: Version,
	/// 包来源。registry 包为 `registry+https://registry.npmjs.org`，
	/// git 包为 `git+https://github.com/user/repo#commit`，
	/// 本地包为 `file:../local-pkg` 或 `link:../workspace-pkg`。
	pub source: SmolStr,
	/// SRI 完整性哈希，例如 `sha512-...`。
	#[serde(skip_serializing_if = "Option::is_none")]
	pub integrity: Option<SmolStr>,
	/// 生产依赖列表，每项格式为 `name@version`
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub dependencies: Option<Vec<SmolStr>>,
	/// devDependencies 列表，每项格式为 `name@version`
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub dev_dependencies: Option<Vec<SmolStr>>,
	/// optionalDependencies 列表，每项格式为 `name@version`
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub optional_dependencies: Option<Vec<SmolStr>>,
	/// peerDependencies 列表，每项格式为 `name@version`
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub peer_dependencies: Option<Vec<SmolStr>>,
	/// 是否有可执行文件
	#[serde(skip_serializing_if = "skip_bin")]
	pub bin: Option<bool>,
	/// 引擎兼容性约束，例如 `{"node": ">=18"}`。
	#[serde(skip_serializing_if = "skip_empty_indexmap")]
	pub engines: Option<IndexMap<SmolStr, SmolStr>>,
	/// 操作系统约束列表，前缀 `!` 表示排除。
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub os: Option<Vec<SmolStr>>,
	/// CPU 架构约束列表，前缀 `!` 表示排除。
	#[serde(skip_serializing_if = "skip_empty_vec")]
	pub cpu: Option<Vec<SmolStr>>,
}

#[cfg(test)]
mod tests {
	use super::*;

	fn de<T: serde::de::DeserializeOwned>(toml_str: &str) -> T {
		toml::from_str(toml_str).expect("failed to deserialize")
	}

	#[test]
	fn parse_minimal_package() {
		let pkg: Package = de(r#"
name = "lodash"
version = "4.17.21"
source = "registry+https://registry.npmjs.org"
"#);
		assert_eq!(pkg.name.as_str(), "lodash");
		assert_eq!(pkg.version.to_string(), "4.17.21");
		assert_eq!(pkg.source.as_str(), "registry+https://registry.npmjs.org");
		assert!(pkg.integrity.is_none());
		assert!(pkg.dependencies.is_none());
		assert!(pkg.dev_dependencies.is_none());
	}

	#[test]
	fn parse_full_package() {
		let pkg: Package = de(r#"
name = "express"
version = "4.18.2"
source = "registry+https://registry.npmjs.org"
integrity = "sha512-abc123"
dependencies = ["accepts@1.3.8", "body-parser@1.20.1"]
devDependencies = ["typescript@^5.0.0", "jest@^29.0.0"]
optionalDependencies = ["fsevents@^2.0.0"]
peerDependencies = ["react@^18.0.0"]
"#);
		assert_eq!(pkg.name.as_str(), "express");
		assert_eq!(pkg.source.as_str(), "registry+https://registry.npmjs.org");
		assert_eq!(pkg.integrity.expect("integrity").as_str(), "sha512-abc123");
		let deps = pkg.dependencies.expect("dependencies");
		assert_eq!(deps.len(), 2);
		assert_eq!(deps[0].as_str(), "accepts@1.3.8");
		let dev = pkg.dev_dependencies.expect("dev");
		assert_eq!(dev.len(), 2);
		assert_eq!(dev[0].as_str(), "typescript@^5.0.0");
		assert_eq!(pkg.peer_dependencies.expect("peer")[0].as_str(), "react@^18.0.0");
	}

	#[test]
	fn parse_git_source_package() {
		let pkg: Package = de(r#"
name = "my-fork"
version = "1.0.0"
source = "git+https://github.com/user/fork#abcdef1234567890"
"#);
		assert_eq!(
			pkg.source.as_str(),
			"git+https://github.com/user/fork#abcdef1234567890"
		);
	}

	#[test]
	fn parse_package_with_bin_and_engines() {
		let pkg: Package = de(r#"
name = "typescript"
version = "5.3.3"
source = "registry+https://registry.npmjs.org"
integrity = "sha512-xyz"
bin = true

[engines]
node = ">=14.17"
"#);
		assert!(pkg.bin.is_some());
		let engines = pkg.engines.expect("engines");
		assert_eq!(engines.get("node").expect("node").as_str(), ">=14.17");
	}

	#[test]
	fn roundtrip_package() {
		let pkg: Package = de(r#"
name = "lodash"
version = "4.17.21"
source = "registry+https://registry.npmjs.org"
integrity = "sha512-abc"
dependencies = ["dep-a@1.0.0"]
"#);
		let serialized = toml::to_string(&pkg).expect("serialize");
		let reparsed: Package = de(&serialized);
		assert_eq!(reparsed.name.as_str(), "lodash");
		assert_eq!(reparsed.version.to_string(), "4.17.21");
		assert_eq!(
			reparsed.source.as_str(),
			"registry+https://registry.npmjs.org"
		);
	}

	#[test]
	fn parse_package_with_os_and_cpu() {
		let pkg: Package = de(r#"
name = "fsevents"
version = "2.3.3"
source = "registry+https://registry.npmjs.org"
os = ["darwin"]
cpu = ["x64", "arm64"]
optionalDependencies = ["fsevents-native@^2.0.0"]
"#);
		let optional = pkg.optional_dependencies.expect("optional");
		assert_eq!(optional[0].as_str(), "fsevents-native@^2.0.0");
		let os = pkg.os.expect("os");
		assert_eq!(os.len(), 1);
		assert_eq!(os[0].as_str(), "darwin");
		let cpu = pkg.cpu.expect("cpu");
		assert_eq!(cpu.len(), 2);
	}

	#[test]
	fn scoped_package_name() {
		let pkg: Package = de(r#"
name = "@types/node"
version = "20.11.0"
source = "registry+https://registry.npmjs.org"
"#);
		assert_eq!(pkg.name.as_str(), "@types/node");
	}

	#[test]
	fn serialize_skips_empty_vec_fields() {
		let pkg = Package {
			name: "empty-deps".into(),
			version: "1.0.0".parse().unwrap(),
			source: "registry+https://registry.npmjs.org".into(),
			integrity: None,
			dependencies: Some(vec![]),
			dev_dependencies: Some(vec![]),
			optional_dependencies: Some(vec![]),
			peer_dependencies: Some(vec![]),
			bin: None,
			engines: None,
			os: Some(vec![]),
			cpu: Some(vec![]),
		};
		let serialized = toml::to_string(&pkg).expect("serialize");
		assert!(!serialized.contains("dependencies"));
		assert!(!serialized.contains("devDependencies"));
		assert!(!serialized.contains("optionalDependencies"));
		assert!(!serialized.contains("peerDependencies"));
		assert!(!serialized.contains("os"));
		assert!(!serialized.contains("cpu"));
	}

	#[test]
	fn serialize_includes_present_fields() {
		let pkg = Package {
			name: "express".into(),
			version: "4.18.2".parse().unwrap(),
			source: "registry+https://registry.npmjs.org".into(),
			integrity: None,
			dependencies: None,
			dev_dependencies: None,
			optional_dependencies: None,
			peer_dependencies: None,
			bin: Some(true),
			engines: None,
			os: None,
			cpu: None,
		};
		let serialized = toml::to_string(&pkg).expect("serialize");
		assert!(serialized.contains("source"));
		assert!(serialized.contains("bin = true"));
	}

	#[test]
	fn serialize_skips_empty_engines() {
		let pkg = Package {
			name: "no-engines".into(),
			version: "1.0.0".parse().unwrap(),
			source: "registry+https://registry.npmjs.org".into(),
			integrity: None,
			dependencies: None,
			dev_dependencies: None,
			optional_dependencies: None,
			peer_dependencies: None,
			bin: None,
			engines: Some(IndexMap::new()),
			os: None,
			cpu: None,
		};
		let serialized = toml::to_string(&pkg).expect("serialize");
		assert!(!serialized.contains("engines ="));
	}

	#[test]
	fn serialize_skips_bin_false() {
		let pkg = Package {
			name: "no-bin".into(),
			version: "1.0.0".parse().unwrap(),
			source: "registry+https://registry.npmjs.org".into(),
			integrity: None,
			dependencies: None,
			dev_dependencies: None,
			optional_dependencies: None,
			peer_dependencies: None,
			bin: Some(false),
			engines: None,
			os: None,
			cpu: None,
		};
		let serialized = toml::to_string(&pkg).expect("serialize");
		assert!(!serialized.contains("bin ="));
	}

	#[test]
	fn parse_empty_optional_fields() {
		let pkg: Package = de(r#"
name = "minimal"
version = "1.0.0"
source = "registry+https://registry.npmjs.org"
dependencies = []
devDependencies = []
optionalDependencies = []
peerDependencies = []
os = []
cpu = []
"#);
		assert!(pkg.dependencies.is_some());
		assert!(pkg.dev_dependencies.is_some());
		assert!(pkg.optional_dependencies.is_some());
		assert!(pkg.peer_dependencies.is_some());
		assert!(pkg.os.is_some());
		assert!(pkg.cpu.is_some());
	}

	#[test]
	fn dependencies_list_allows_duplicates() {
		let pkg: Package = de(r#"
name = "test"
version = "1.0.0"
source = "registry+https://registry.npmjs.org"
dependencies = ["react@18.0.0", "react@17.0.0", "react@18.0.0"]
"#);
		let deps = pkg.dependencies.expect("dependencies");
		assert_eq!(deps.len(), 3);
		assert_eq!(deps[0].as_str(), "react@18.0.0");
		assert_eq!(deps[1].as_str(), "react@17.0.0");
		assert_eq!(deps[2].as_str(), "react@18.0.0");
	}

	#[test]
	fn parse_file_source() {
		let pkg: Package = de(r#"
name = "local-pkg"
version = "1.0.0"
source = "file:../packages/local"
"#);
		assert_eq!(pkg.source.as_str(), "file:../packages/local");
	}

	#[test]
	fn parse_link_source() {
		let pkg: Package = de(r#"
name = "workspace-pkg"
version = "1.0.0"
source = "link:../workspace/pkg"
"#);
		assert_eq!(pkg.source.as_str(), "link:../workspace/pkg");
	}

	#[test]
	fn parse_exclude_os() {
		let pkg: Package = de(r#"
name = "windows-only"
version = "1.0.0"
source = "registry+https://registry.npmjs.org"
os = ["!linux", "!darwin"]
"#);
		let os = pkg.os.expect("os");
		assert_eq!(os.len(), 2);
		assert_eq!(os[0].as_str(), "!linux");
		assert_eq!(os[1].as_str(), "!darwin");
	}

	#[test]
	fn parse_exclude_cpu() {
		let pkg: Package = de(r#"
name = "x64-only"
version = "1.0.0"
source = "registry+https://registry.npmjs.org"
cpu = ["!arm", "!arm64"]
"#);
		let cpu = pkg.cpu.expect("cpu");
		assert_eq!(cpu.len(), 2);
		assert_eq!(cpu[0].as_str(), "!arm");
		assert_eq!(cpu[1].as_str(), "!arm64");
	}
}
