use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use url::Url;
use indexmap::IndexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageBugs {
	Url(Url),
	Full {
		#[serde(skip_serializing_if = "Option::is_none")]
		url: Option<Url>,
		#[serde(skip_serializing_if = "Option::is_none")]
		email: Option<SmolStr>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonFull {
	pub name: SmolStr,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub email: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Person {
	Short(SmolStr),
	Full(PersonFull),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageRepository {
	Short(SmolStr),
	Full {
		r#type: SmolStr,
		url: SmolStr,
		#[serde(skip_serializing_if = "Option::is_none")]
		directory: Option<SmolStr>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFunding {
	pub r#type: SmolStr,
	pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FundingEntry {
	Url(Url),
	Full(PackageFunding),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageFundings {
	One(FundingEntry),
	Many(Vec<FundingEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageBin {
	Path(SmolStr),
	Map(IndexMap<SmolStr, SmolStr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageMan {
	One(SmolStr),
	Many(Vec<SmolStr>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDist {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub shasum: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tarball: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDirectories {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bin: Option<SmolStr>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub man: Option<SmolStr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageExportsEntry {
	Path(SmolStr),
	Conditions(IndexMap<SmolStr, PackageExportsEntry>),
	Array(Vec<PackageExportsEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageExports {
	Path(SmolStr),
	Conditions(IndexMap<SmolStr, PackageExportsEntry>),
	Array(Vec<PackageExportsEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PackageImports {
	Conditions(IndexMap<SmolStr, PackageExportsEntry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
	Commonjs,
	Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageOverride(pub IndexMap<SmolStr, OverrideValue>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OverrideValue {
	Version(SmolStr),
	Nested(IndexMap<SmolStr, OverrideValue>),
}

#[cfg(test)]
mod tests {
	use super::*;

	fn de<T: serde::de::DeserializeOwned>(json: &str) -> T {
		serde_json::from_str(json).unwrap()
	}

	#[test]
	fn bugs_url() {
		let b: PackageBugs = de(r#""https://github.com/user/repo/issues""#);
		match b {
			PackageBugs::Url(url) => assert_eq!(url.as_str(), "https://github.com/user/repo/issues"),
			_ => panic!("expected Url variant"),
		}
	}

	#[test]
	fn bugs_full() {
		let b: PackageBugs = de(r#"{"url": "https://github.com/u/r/issues", "email": "a@b.com"}"#);
		match b {
			PackageBugs::Full { url, email } => {
				assert!(url.is_some());
				assert_eq!(email.unwrap().as_str(), "a@b.com");
			}
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn bugs_full_partial() {
		let b: PackageBugs = de(r#"{"email": "a@b.com"}"#);
		match b {
			PackageBugs::Full { url, email } => {
				assert!(url.is_none());
				assert_eq!(email.unwrap().as_str(), "a@b.com");
			}
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn person_short() {
		let p: Person = de(r#""John Doe <john@example.com>""#);
		match p {
			Person::Short(s) => assert!(s.contains("John Doe")),
			_ => panic!("expected Short variant"),
		}
	}

	#[test]
	fn person_full() {
		let p: Person = de(r#"{"name": "John", "email": "j@e.com", "url": "https://example.com"}"#);
		match p {
			Person::Full(f) => {
				assert_eq!(f.name.as_str(), "John");
				assert_eq!(f.email.unwrap().as_str(), "j@e.com");
				assert!(f.url.is_some());
			}
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn person_full_name_only() {
		let p: Person = de(r#"{"name": "John"}"#);
		match p {
			Person::Full(f) => {
				assert_eq!(f.name.as_str(), "John");
				assert!(f.email.is_none());
				assert!(f.url.is_none());
			}
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn repository_short() {
		let r: PackageRepository = de(r#""github:user/repo""#);
		match r {
			PackageRepository::Short(s) => assert_eq!(s.as_str(), "github:user/repo"),
			_ => panic!("expected Short variant"),
		}
	}

	#[test]
	fn repository_full() {
		let r: PackageRepository = de(r#"{"type": "git", "url": "https://github.com/u/r.git", "directory": "packages/a"}"#);
		match r {
			PackageRepository::Full { r#type, url, directory } => {
				assert_eq!(r#type.as_str(), "git");
				assert_eq!(url.as_str(), "https://github.com/u/r.git");
				assert_eq!(directory.unwrap().as_str(), "packages/a");
			}
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn repository_full_no_directory() {
		let r: PackageRepository = de(r#"{"type": "git", "url": "https://github.com/u/r.git"}"#);
		match r {
			PackageRepository::Full { directory, .. } => assert!(directory.is_none()),
			_ => panic!("expected Full variant"),
		}
	}

	#[test]
	fn funding_single_url() {
		let f: PackageFundings = de(r#""https://example.com/fund""#);
		match f {
			PackageFundings::One(FundingEntry::Url(url)) => {
				assert_eq!(url.as_str(), "https://example.com/fund");
			}
			_ => panic!("expected One(Url)"),
		}
	}

	#[test]
	fn funding_single_object() {
		let f: PackageFundings = de(r#"{"type": "github", "url": "https://github.com/sponsors/u"}"#);
		match f {
			PackageFundings::One(FundingEntry::Full(pf)) => {
				assert_eq!(pf.r#type.as_str(), "github");
			}
			_ => panic!("expected One(Full)"),
		}
	}

	#[test]
	fn funding_array() {
		let f: PackageFundings = de(r#"["https://a.com", {"type": "patreon", "url": "https://b.com"}]"#);
		match f {
			PackageFundings::Many(v) => assert_eq!(v.len(), 2),
			_ => panic!("expected Many"),
		}
	}

	#[test]
	fn bin_path() {
		let b: PackageBin = de(r#""./cli.js""#);
		match b {
			PackageBin::Path(s) => assert_eq!(s.as_str(), "./cli.js"),
			_ => panic!("expected Path variant"),
		}
	}

	#[test]
	fn bin_map() {
		let b: PackageBin = de(r#"{"cmd1": "./a.js", "cmd2": "./b.js"}"#);
		match b {
			PackageBin::Map(m) => {
				assert_eq!(m.len(), 2);
				assert_eq!(m.get("cmd1").unwrap().as_str(), "./a.js");
			}
			_ => panic!("expected Map variant"),
		}
	}

	#[test]
	fn man_one() {
		let m: PackageMan = de(r#""./man/foo.1""#);
		match m {
			PackageMan::One(s) => assert_eq!(s.as_str(), "./man/foo.1"),
			_ => panic!("expected One variant"),
		}
	}

	#[test]
	fn man_many() {
		let m: PackageMan = de(r#"["./man/foo.1", "./man/bar.1"]"#);
		match m {
			PackageMan::Many(v) => assert_eq!(v.len(), 2),
			_ => panic!("expected Many variant"),
		}
	}

	#[test]
	fn directories_full() {
		let d: PackageDirectories = de(r#"{"bin": "./bin", "man": "./man"}"#);
		assert_eq!(d.bin.unwrap().as_str(), "./bin");
		assert_eq!(d.man.unwrap().as_str(), "./man");
	}

	#[test]
	fn directories_partial() {
		let d: PackageDirectories = de(r#"{"bin": "./bin"}"#);
		assert!(d.bin.is_some());
		assert!(d.man.is_none());
	}

	#[test]
	fn exports_path() {
		let e: PackageExports = de(r#""./index.js""#);
		match e {
			PackageExports::Path(s) => assert_eq!(s.as_str(), "./index.js"),
			_ => panic!("expected Path variant"),
		}
	}

	#[test]
	fn exports_conditions() {
		let e: PackageExports = de(r#"{".": {"import": "./a.mjs", "require": "./a.cjs"}}"#);
		match e {
			PackageExports::Conditions(m) => {
				assert!(m.contains_key("."));
			}
			_ => panic!("expected Conditions variant"),
		}
	}

	#[test]
	fn exports_nested_conditions() {
		let e: PackageExports = de(r#"{".": {"node": {"import": "./a.mjs", "require": "./a.cjs"}, "default": "./a.js"}}"#);
		match e {
			PackageExports::Conditions(m) => {
				let dot = m.get(".").unwrap();
				match dot {
					PackageExportsEntry::Conditions(inner) => {
						assert!(inner.contains_key("node"));
						assert!(inner.contains_key("default"));
					}
					_ => panic!("expected nested Conditions"),
				}
			}
			_ => panic!("expected Conditions variant"),
		}
	}

	#[test]
	fn package_type_module() {
		let t: PackageType = de(r#""module""#);
		assert!(matches!(t, PackageType::Module));
	}

	#[test]
	fn package_type_commonjs() {
		let t: PackageType = de(r#""commonjs""#);
		assert!(matches!(t, PackageType::Commonjs));
	}

	#[test]
	fn override_version() {
		let o: OverrideValue = de(r#""1.0.0""#);
		match o {
			OverrideValue::Version(v) => assert_eq!(v.as_str(), "1.0.0"),
			_ => panic!("expected Version variant"),
		}
	}

	#[test]
	fn override_nested() {
		let o: OverrideValue = de(r#"{"baz": "2.0.0"}"#);
		match o {
			OverrideValue::Nested(m) => {
				match m.get("baz").unwrap() {
					OverrideValue::Version(v) => assert_eq!(v.as_str(), "2.0.0"),
					_ => panic!("expected nested Version"),
				}
			}
			_ => panic!("expected Nested variant"),
		}
	}

	#[test]
	fn roundtrip_bugs_url() {
		let b: PackageBugs = de(r#""https://github.com/user/repo/issues""#);
		let json = serde_json::to_string(&b).unwrap();
		let b2: PackageBugs = de(&json);
		match b2 {
			PackageBugs::Url(_) => {}
			_ => panic!("roundtrip failed"),
		}
	}

	#[test]
	fn roundtrip_person_full() {
		let p: Person = de(r#"{"name": "A", "email": "a@b.com"}"#);
		let json = serde_json::to_string(&p).unwrap();
		let p2: Person = de(&json);
		match p2 {
			Person::Full(f) => assert_eq!(f.name.as_str(), "A"),
			_ => panic!("roundtrip failed"),
		}
	}
}