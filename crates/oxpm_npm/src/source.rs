use oxpm_source::Source;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpmSource {
    registry_url: Url,
    name: SmolStr,
}

impl NpmSource {
    pub fn new(registry_url: Url, name: SmolStr) -> Self {
        Self { registry_url, name }
    }

    pub fn registry_url(&self) -> &Url {
        &self.registry_url
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Source for NpmSource {
    fn name(&self) -> &'static str {
        "Npm"
    }

    fn as_str(&self) -> SmolStr {
        let base = self.registry_url.as_str().trim_end_matches('/');
        SmolStr::new(format!("registry+{base}"))
    }
}

impl std::fmt::Display for NpmSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_as_str() {
        let source = NpmSource::new(
            Url::parse("https://registry.npmjs.org/").unwrap(),
            SmolStr::new("lodash"),
        );
        assert_eq!(source.as_str().as_str(), "registry+https://registry.npmjs.org");
    }

    #[test]
    fn source_trailing_slash() {
        let source = NpmSource::new(
            Url::parse("https://custom.registry.com/npm/").unwrap(),
            SmolStr::new("pkg"),
        );
        assert_eq!(
            source.as_str().as_str(),
            "registry+https://custom.registry.com/npm"
        );
    }
}
