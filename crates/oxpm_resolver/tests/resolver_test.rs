
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
	PathBuf::from(env!("CARGO_MANIFEST_DIR"))
		.join("tests")
		.join("fixtures")
		.join(name)
}


#[tokio::test]
#[ignore]
async fn test_resolve_from_package_json() {
	let pkg_path = fixture_path("package.json");
	let pkg = oxpm_package_json::PackageJson::load_from_path(&pkg_path).unwrap();

	let resolver = oxpm_resolver::Resolver::new().unwrap();
	let _tree = resolver.with_package_json(pkg).resolve().await;
}

#[test]
fn test_load_package_with_deps() {
	let pkg_path = fixture_path("package.json");
	let pkg = oxpm_package_json::PackageJson::load_from_path(&pkg_path).unwrap();

	assert_eq!(pkg.name.as_deref(), Some("test-app"));
	assert!(pkg.dependencies.is_some());
	assert!(pkg.dev_dependencies.is_some());
}

#[test]
fn test_empty_dependencies() {
	let pkg = r#"{
		"name": "empty-app",
		"version": "1.0.0"
	}"#;
	let pkg = oxpm_package_json::PackageJson::load_from_str(pkg).unwrap();

	assert_eq!(pkg.name.as_deref(), Some("empty-app"));
	assert!(pkg.dependencies.is_none());
}