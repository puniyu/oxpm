use oxpm_npm::lockfile::{LockFile, LockfileVersion};
use oxpm_npm::LockFileError;

#[test]
fn lockfile_version_as_u32() {
	assert_eq!(LockfileVersion::V3.as_u32(), 3);
}

#[test]
fn lockfile_version_try_from_v1() {
	assert!(matches!(
		LockfileVersion::try_from(1u64),
		Err(LockFileError::UnsupportedLockfileVersion(1))
	));
}

#[test]
fn lockfile_version_try_from_v2() {
	assert!(matches!(
		LockfileVersion::try_from(2u64),
		Err(LockFileError::UnsupportedLockfileVersion(2))
	));
}

#[test]
fn lockfile_version_try_from_v3() {
	assert_eq!(LockfileVersion::try_from(3u64).unwrap(), LockfileVersion::V3);
}

#[test]
fn lockfile_version_try_from_unknown() {
	assert!(matches!(
		LockfileVersion::try_from(99u64),
		Err(LockFileError::UnsupportedLockfileVersion(99))
	));
}

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
			.as_ref()
			.unwrap()
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
	assert_eq!(
		pkg.source.to_source_string().as_str(),
		"git+https://example.com/types-node#abc"
	);
	assert_eq!(pkg.dependencies.as_ref().unwrap()[0].as_str(), "undici-types@~5.26.4");
	assert!(pkg.bin.is_some());
	assert_eq!(pkg.engines.as_ref().unwrap().get("node").unwrap().as_str(), ">=18");
	assert_eq!(pkg.os.as_ref().unwrap()[0].as_str(), "linux");
	assert_eq!(pkg.cpu.as_ref().unwrap()[0].as_str(), "x64");
}

#[test]
fn convert_missing_package_version_returns_error() {
	let json = r#"{
		"lockfileVersion": 3,
		"packages": {
			"node_modules/lodash": {}
		}
	}"#;

	let lockfile = LockFile::load_from_str(json).unwrap();
	assert!(matches!(
		lockfile.to_oxpm_lockfile(),
		Err(LockFileError::MissingPackageVersion { .. })
	));
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
	assert!(matches!(err, LockFileError::Parse(_)));
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
	let json = r#"{
		"lockfileVersion": 3,
		"packages": {
			"weird/path": {
				"version": "1.0.0"
			}
		}
	}"#;

	let lockfile = LockFile::load_from_str(json).unwrap();
	assert!(matches!(
		lockfile.to_oxpm_lockfile(),
		Err(LockFileError::InvalidPackagePath(_))
	));
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
	let json = r#"{
		"lockfileVersion": 3,
		"packages": {
			"node_modules/@types/node": {
				"version": "1.0.0"
			}
		}
	}"#;
	let lockfile = LockFile::load_from_str(json).unwrap();
	let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
	assert_eq!(oxpm_lockfile.packages[0].name.as_str(), "@types/node");
}

#[test]
fn resolve_source_non_http_keeps_original() {
	let json = r#"{
		"lockfileVersion": 3,
		"packages": {
			"node_modules/pkg": {
				"version": "1.0.0",
				"resolved": "git+https://example.com/repo#abc"
			}
		}
	}"#;
	let lockfile = LockFile::load_from_str(json).unwrap();
	let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
	assert_eq!(
		oxpm_lockfile.packages[0]
			.source
			.to_source_string()
			.as_str(),
		"git+https://example.com/repo#abc"
	);
}

#[test]
fn resolve_source_http_extracts_registry_origin() {
	let json = r#"{
		"lockfileVersion": 3,
		"packages": {
			"node_modules/pkg": {
				"version": "1.0.0",
				"resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz"
			}
		}
	}"#;
	let lockfile = LockFile::load_from_str(json).unwrap();
	let oxpm_lockfile = lockfile.to_oxpm_lockfile().unwrap();
	assert_eq!(
		oxpm_lockfile.packages[0]
			.source
			.to_source_string()
			.as_str(),
		"registry+https://registry.npmjs.org"
	);
}