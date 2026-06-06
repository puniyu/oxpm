use reqwest::Client;
use smol_str::SmolStr;
use url::Url;

mod error;
pub use error::Error;
mod types;
pub use types::*;

pub type Result<T> = std::result::Result<T, Error>;

const USER_AGENT: &str = concat!("fnpm", "/", env!("VERSION"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryAuth {
    BearerToken(SmolStr),
    BasicToken(SmolStr),
    Basic { username: SmolStr, password: SmolStr },
}

pub struct Registry {
    client: Client,
    registry_url: Url,
    auth: Option<RegistryAuth>,
}

impl Registry {
    pub fn new(registry_url: Url) -> Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build().map_err(Error::Http)?;
        Ok(Self {
            client,
            registry_url,
            auth: None,
        })
    }

    pub fn auth(mut self, auth: RegistryAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.auth.as_ref() {
            Some(RegistryAuth::BearerToken(token)) => {
                req.header(http::header::AUTHORIZATION, format!("Bearer {token}"))
            }
            Some(RegistryAuth::BasicToken(token)) => {
                req.header(http::header::AUTHORIZATION, format!("Basic {token}"))
            }
            Some(RegistryAuth::Basic { username, password }) => {
                req.basic_auth(username.as_str(), Some(password.as_str()))
            }
            None => req,
        }
    }

    pub async fn package(&self, name: &str) -> Result<Package> {
        use http::header;
        let base = self.registry_url.as_str().trim_end_matches('/');
        let url = format!("{base}/{name}");
        let url = Url::parse(&url).map_err(|_| Error::PackageNotFound(SmolStr::new(name)))?;

        let response = self
            .apply_auth(self.client.get(url))
            .header(header::ACCEPT, mime::APPLICATION_JSON.as_ref())
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::PackageNotFound(SmolStr::new(name)));
        }

        let package = response.error_for_status()?.json::<Package>().await?;
        Ok(package)
    }

    pub async fn package_version(&self, name: &str, version: &str) -> Result<PackageVersion> {
        let package = self.package(name).await?;
        package
            .versions
            .get(version)
            .cloned()
            .ok_or_else(|| Error::VersionNotFound {
                name: SmolStr::new(name),
                version: SmolStr::new(version),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_registry_url() -> Url {
        Url::parse("https://example.com/npm/").unwrap()
    }

    #[test]
    fn new_preserves_registry_url() {
        let registry = Registry::new(Url::parse("https://example.com/npm/registry/").unwrap()).unwrap();
        assert!(registry.registry_url.as_str().contains("/npm/registry"));
    }

    #[test]
    fn auth_builder_sets_bearer_token() {
        let registry = Registry::new(example_registry_url())
            .unwrap()
            .auth(RegistryAuth::BearerToken("token-123".into()));
        assert_eq!(registry.auth, Some(RegistryAuth::BearerToken("token-123".into())));
    }

    #[test]
    fn auth_builder_sets_basic_token() {
        let registry = Registry::new(example_registry_url())
            .unwrap()
            .auth(RegistryAuth::BasicToken("dXNlcjpzZWNyZXQ=".into()));
        assert_eq!(registry.auth, Some(RegistryAuth::BasicToken("dXNlcjpzZWNyZXQ=".into())));
    }

    #[test]
    fn auth_builder_sets_basic_auth() {
        let registry = Registry::new(example_registry_url())
            .unwrap()
            .auth(RegistryAuth::Basic {
                username: "user".into(),
                password: "secret".into(),
            });
        assert!(matches!(registry.auth, Some(RegistryAuth::Basic { .. })));
    }
}