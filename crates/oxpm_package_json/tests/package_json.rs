use oxpm_package_json::{PackageJson, PackageType, PackageExports, OverrideValue, Error};

#[test]
fn parse_minimal() {
    let pkg = PackageJson::load_from_str(r#"{"name": "my-pkg", "version": "1.0.0"}"#).unwrap();
    assert_eq!(pkg.name.unwrap().as_str(), "my-pkg");
    assert_eq!(pkg.version.unwrap().to_string(), "1.0.0");
}

#[test]
fn parse_empty_object() {
    let pkg = PackageJson::load_from_str("{}").unwrap();
    assert!(pkg.name.is_none());
    assert!(pkg.version.is_none());
    assert!(pkg.dependencies.is_none());
    assert!(pkg.keywords.is_empty());
}

#[test]
fn parse_invalid_json() {
    let err = PackageJson::load_from_str("not json").unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
}

#[test]
fn parse_all_dependency_types() {
    let json = r#"{
        "dependencies": {"lodash": "^4.17.21"},
        "devDependencies": {"jest": "^29.0.0"},
        "peerDependencies": {"react": ">=18"},
        "optionalDependencies": {"fsevents": "^2.3.0"},
        "peerDependenciesMeta": {"react": {"optional": false}},
        "bundleDependencies": ["lodash"]
    }"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.dependencies.unwrap().get("lodash").unwrap().as_str(), "^4.17.21");
    assert_eq!(pkg.dev_dependencies.unwrap().len(), 1);
    assert_eq!(pkg.peer_dependencies.unwrap().len(), 1);
    assert_eq!(pkg.optional_dependencies.unwrap().len(), 1);
    assert_eq!(pkg.peer_dependencies_meta.unwrap().len(), 1);
    assert_eq!(pkg.bundle_dependencies.len(), 1);
}

#[test]
fn parse_scripts() {
    let json = r#"{"scripts": {"build": "tsc", "test": "jest", "lint": "eslint ."}}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    let scripts = pkg.scripts.unwrap();
    assert_eq!(scripts.len(), 3);
    assert_eq!(scripts.get("build").unwrap().as_str(), "tsc");
}

#[test]
fn parse_engines_os_cpu() {
    let json = r#"{"engines": {"node": ">=18"}, "os": ["!win32", "linux"], "cpu": ["x64", "arm64"]}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.engines.unwrap().get("node").unwrap().as_str(), ">=18");
    assert_eq!(pkg.os.len(), 2);
    assert_eq!(pkg.cpu.len(), 2);
}

#[test]
fn parse_private_and_type() {
    let json = r#"{"private": true, "type": "module"}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.private, Some(true));
    assert!(matches!(pkg.module_type.unwrap(), PackageType::Module));
}

#[test]
fn parse_workspaces() {
    let json = r#"{"workspaces": ["packages/*", "apps/*"]}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.workspaces.len(), 2);
}

#[test]
fn parse_overrides() {
    let json = r#"{"overrides": {"foo": "1.0.0", "bar": {"baz": "2.0.0"}}}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    let overrides = pkg.overrides.unwrap();
    assert!(matches!(overrides.get("foo").unwrap(), OverrideValue::Version(_)));
    assert!(matches!(overrides.get("bar").unwrap(), OverrideValue::Nested(_)));
}

#[test]
fn parse_exports_and_imports() {
    let json = r##"{
        "exports": {".": {"import": "./a.mjs", "require": "./a.cjs"}},
        "imports": {"#utils": "./src/utils.js"}
    }"##;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert!(matches!(pkg.exports.unwrap(), PackageExports::Conditions(_)));
    assert_eq!(pkg.imports.unwrap().len(), 1);
}

#[test]
fn parse_full_package_json() {
    let json = r##"{
        "name": "@scope/my-package",
        "version": "2.1.0",
        "description": "A full example",
        "keywords": ["test", "example"],
        "homepage": "https://example.com",
        "bugs": {"url": "https://github.com/u/r/issues"},
        "license": "MIT",
        "author": {"name": "Test", "email": "t@e.com"},
        "contributors": ["Contrib <c@e.com>"],
        "funding": {"type": "github", "url": "https://github.com/sponsors/u"},
        "files": ["dist"],
        "main": "./dist/index.js",
        "browser": "./dist/browser.js",
        "bin": {"cli": "./dist/cli.js"},
        "man": "./man/doc.1",
        "directories": {"bin": "./bin"},
        "repository": {"type": "git", "url": "https://github.com/u/r.git"},
        "scripts": {"build": "tsc"},
        "dependencies": {"lodash": "^4.17.21"},
        "devDependencies": {"typescript": "^5.0.0"},
        "engines": {"node": ">=18"},
        "os": ["linux"],
        "cpu": ["x64"],
        "private": false,
        "type": "module",
        "exports": {".": {"import": "./dist/index.mjs"}},
        "imports": {"#utils": "./src/utils.js"},
        "workspaces": ["packages/*"]
    }"##;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.name.unwrap().as_str(), "@scope/my-package");
    assert_eq!(pkg.keywords.len(), 2);
    assert_eq!(pkg.license.unwrap().as_str(), "MIT");
    assert_eq!(pkg.private, Some(false));
    assert!(pkg.exports.is_some());
    assert!(pkg.imports.is_some());
    assert_eq!(pkg.workspaces.len(), 1);
}

#[test]
fn roundtrip_serialize() {
    let json = r#"{"name":"my-pkg","version":"1.0.0","private":true,"type":"module"}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    let output = serde_json::to_string(&pkg).unwrap();
    let reparsed = PackageJson::load_from_str(&output).unwrap();
    assert_eq!(reparsed.name.unwrap().as_str(), "my-pkg");
    assert_eq!(reparsed.private, Some(true));
    assert!(matches!(reparsed.module_type.unwrap(), PackageType::Module));
}

#[test]
fn camel_case_rename() {
    let json = r#"{"devDependencies": {"a": "1"}, "peerDependenciesMeta": {"b": {"optional": true}}, "publishConfig": {"registry": "https://r.com"}}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert!(pkg.dev_dependencies.is_some());
    assert!(pkg.peer_dependencies_meta.is_some());
    assert!(pkg.publish_config.is_some());
}

#[test]
fn unknown_fields_ignored() {
    let json = r#"{"name": "test", "someCustomField": 42, "x-custom": true}"#;
    let pkg = PackageJson::load_from_str(json).unwrap();
    assert_eq!(pkg.name.unwrap().as_str(), "test");
}

#[test]
fn load_from_path() {
    let pkg = PackageJson::load_from_path("tests/fixtures/package.json").unwrap();
    assert_eq!(pkg.name.as_ref().unwrap().as_str(), "test-pkg");
    assert_eq!(pkg.version.as_ref().unwrap().to_string(), "1.0.0");
}

#[test]
fn load_from_path_not_found() {
    let err = PackageJson::load_from_path("tests/fixtures/not-exist.json").unwrap_err();
    assert!(matches!(err, Error::Io(_)));
}

#[test]
fn load_from_path_invalid_json() {
    let err = PackageJson::load_from_path("tests/fixtures/invalid.json").unwrap_err();
    assert!(matches!(err, Error::Parse(_)));
}