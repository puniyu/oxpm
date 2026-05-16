use reqwest::Client;
use url::Url;

mod error;
pub use error::Error;
mod types;
pub use types::*;

pub type Result<T> = std::result::Result<T, Error>;

const NPM_REGISTRY: &str = "https://registry.npmjs.org";
const JSR_REGISTRY: &str = "https://npm.jsr.io";

pub struct Registry {
    client: Client,
    registry_url: Url,
}

impl Registry {
    pub fn npm() -> Result<Self> {
        Self::with_url(Url::parse(NPM_REGISTRY).expect("invalid default npm registry url"))
    }

    pub fn jsr() -> Result<Self> {
        Self::with_url(Url::parse(JSR_REGISTRY).expect("invalid default jsr registry url"))
    }

    pub fn with_url(registry_url: Url) -> Result<Self> {
        let client = Client::builder().build().map_err(Error::Http)?;
        Ok(Self { client, registry_url })
    }

    pub async fn package(&self, name: &str) -> Result<Package> {
        let base = self.registry_url.as_str().trim_end_matches('/');
        let url = format!("{base}/{name}");
        let url = Url::parse(&url).map_err(|_| Error::PackageNotFound(name.to_string()))?;

        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::PackageNotFound(name.to_string()));
        }

        let package: Package = response.error_for_status()?.json().await?;
        Ok(package)
    }

    pub async fn package_version(&self, name: &str, version: &str) -> Result<PackageVersion> {
        let package = self.package(name).await?;
        package
            .versions
            .get(version)
            .cloned()
            .ok_or_else(|| Error::VersionNotFound {
                name: name.to_string(),
                version: version.to_string(),
            })
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::npm().expect("failed to create default Registry")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npm_registry_url() {
        let r = Registry::npm().unwrap();
        assert!(r.registry_url.as_str().contains("registry.npmjs.org"));
    }

    #[test]
    fn jsr_registry_url() {
        let r = Registry::jsr().unwrap();
        assert!(r.registry_url.as_str().contains("npm.jsr.io"));
    }

    #[test]
    fn with_url_preserves_path() {
        let url = Url::parse("https://example.com/npm/registry/").unwrap();
        let r = Registry::with_url(url).unwrap();
        assert!(r.registry_url.as_str().contains("/npm/registry"));
    }

    #[test]
    fn default_is_npm() {
        let r = Registry::default();
        assert!(r.registry_url.as_str().contains("registry.npmjs.org"));
    }

    #[tokio::test]
    async fn package_not_found() {
        let r = Registry::npm().unwrap();
        let err = r.package("@oxpm-test/this-package-does-not-exist-ever").await.unwrap_err();
        assert!(matches!(err, Error::PackageNotFound(_)));
    }

    #[tokio::test]
    async fn package_version_not_found() {
        let r = Registry::npm().unwrap();
        let err = r.package_version("es-toolkit", "0.0.0-nonexistent").await.unwrap_err();
        assert!(matches!(err, Error::VersionNotFound { .. }));
    }

    #[tokio::test]
    async fn fetch_package() {
        let r = Registry::npm().unwrap();
        let pkg = r.package("es-toolkit").await.unwrap();
        assert_eq!(pkg.name.as_str(), "es-toolkit");
        assert!(pkg.dist_tags.contains_key("latest"));
        assert!(!pkg.versions.is_empty());
    }

    #[tokio::test]
    async fn fetch_package_version() {
        let r = Registry::npm().unwrap();
        let v = r.package_version("es-toolkit", "1.27.0").await.unwrap();
        assert_eq!(v.name.as_str(), "es-toolkit");
        assert_eq!(v.version.to_string(), "1.27.0");
        assert!(v.dist.tarball.as_str().contains("es-toolkit"));
    }

    #[tokio::test]
    async fn fetch_scoped_package() {
        let r = Registry::npm().unwrap();
        let pkg = r.package("@types/node").await.unwrap();
        assert_eq!(pkg.name.as_str(), "@types/node");
    }
}
