use std::{fs, path::Path, str::FromStr};

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

mod auth;
pub use auth::*;
mod error;
pub use error::Error;
mod registry;
pub use registry::*;

pub(crate) const DEFAULT_NPM_SCOPE: &str = "npm";
pub(crate) const DEFAULT_JSR_SCOPE: &str = "jsr";

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "SmolStr", into = "SmolStr")]
pub enum ScopeType {
	#[default]
	Npm,
	Jsr,
	Other(SmolStr),
}

impl FromStr for ScopeType {
	type Err = std::convert::Infallible;

	#[inline]
	fn from_str(scope: &str) -> std::result::Result<Self, Self::Err> {
		Ok(Self::from(scope))
	}
}

impl From<&str> for ScopeType {
	fn from(scope: &str) -> Self {
		match scope {
			DEFAULT_NPM_SCOPE => Self::Npm,
			DEFAULT_JSR_SCOPE => Self::Jsr,
			_ => Self::Other(scope.into()),
		}
	}
}

impl From<SmolStr> for ScopeType {
	#[inline]
	fn from(scope: SmolStr) -> Self {
		Self::from(scope.as_str())
	}
}

impl From<ScopeType> for SmolStr {
	fn from(scope: ScopeType) -> Self {
		match scope {
			ScopeType::Npm => DEFAULT_NPM_SCOPE.into(),
			ScopeType::Jsr => DEFAULT_JSR_SCOPE.into(),
			ScopeType::Other(scope) => scope,
		}
	}
}

impl ScopeType {
	pub fn as_str(&self) -> &str {
		match self {
			Self::Npm => DEFAULT_NPM_SCOPE,
			Self::Jsr => DEFAULT_JSR_SCOPE,
			Self::Other(scope) => scope.as_str(),
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub registry: RegistryConfig,
	#[serde(default)]
	pub auth: AuthConfig,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Config {
	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
		let content = fs::read_to_string(path)?;
		Self::load_from_str(&content)
	}

	pub fn load_from_str(content: &str) -> Result<Self> {
		if content.trim().is_empty() {
			return Ok(Self::default());
		}

		Ok(toml::from_str(content)?)
	}
}

