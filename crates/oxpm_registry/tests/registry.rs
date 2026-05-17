use oxpm_registry::{Error, Registry};
use url::Url;

fn npm_registry_url() -> Url {
    Url::parse("https://registry.npmjs.org/").unwrap()
}

#[tokio::test]
async fn package_not_found() {
    let registry = Registry::new(npm_registry_url()).unwrap();
    let err = registry
        .package("@fnpm-test/this-package-does-not-exist-ever")
        .await
        .unwrap_err();
    assert!(matches!(err, Error::PackageNotFound(_)));
}

#[tokio::test]
async fn package_version_not_found() {
    let registry = Registry::new(npm_registry_url()).unwrap();
    let err = registry
        .package_version("es-toolkit", "99.99.99")
        .await
        .unwrap_err();
    assert!(matches!(err, Error::VersionNotFound { .. }));
}

#[tokio::test]
async fn fetch_package() {
    let registry = Registry::new(npm_registry_url()).unwrap();
    let pkg = registry.package("es-toolkit").await.unwrap();
    assert_eq!(pkg.name.as_str(), "es-toolkit");
    assert!(pkg.dist_tags.contains_key("latest"));
    assert!(!pkg.versions.is_empty());
}

#[tokio::test]
async fn fetch_package_version() {
    let registry = Registry::new(npm_registry_url()).unwrap();
    let version = registry.package_version("es-toolkit", "1.27.0").await.unwrap();
    assert_eq!(version.name.as_str(), "es-toolkit");
    assert_eq!(version.version.to_string(), "1.27.0");
    assert!(version.dist.tarball.as_str().contains("es-toolkit"));
}

#[tokio::test]
async fn fetch_scoped_package() {
    let registry = Registry::new(npm_registry_url()).unwrap();
    let pkg = registry.package("@types/node").await.unwrap();
    assert_eq!(pkg.name.as_str(), "@types/node");
}