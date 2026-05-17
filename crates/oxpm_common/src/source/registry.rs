use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::SourceError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegistrySource {
	registry_url: Url,
}

impl RegistrySource {
	pub fn new(url: Url) -> Self {
		Self { registry_url: url }
	}

	pub fn url(&self) -> &Url {
		&self.registry_url
	}

	pub fn as_str(&self) -> &str {
		self.registry_url.as_str().trim_end_matches('/')
	}
}

impl std::fmt::Display for RegistrySource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.registry_url)
	}
}

impl AsRef<str> for RegistrySource {
	fn as_ref(&self) -> &str {
		self.registry_url.as_str()
	}
}

impl TryFrom<&str> for RegistrySource {
	type Error = SourceError;

	fn try_from(url: &str) -> Result<Self, Self::Error> {
		Url::parse(url)
			.map(Self::new)
			.map_err(|_| SourceError(url.to_string()))
	}
}

impl TryFrom<String> for RegistrySource {
	type Error = SourceError;

	fn try_from(url: String) -> Result<Self, Self::Error> {
		Self::try_from(url.as_str())
	}
}

impl From<RegistrySource> for Url {
	fn from(source: RegistrySource) -> Url {
		source.registry_url
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn display() {
		let source = RegistrySource::new(Url::parse("https://registry.npmjs.org").unwrap());
		assert_eq!(format!("{}", source), "https://registry.npmjs.org/");
	}

	#[test]
	fn from_str() {
		let source = RegistrySource::try_from("https://registry.npmjs.org").unwrap();
		assert_eq!(source.as_str(), "https://registry.npmjs.org");
	}
}