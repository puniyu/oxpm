use serde::{Deserialize, Deserializer, Serialize};
use smol_str::SmolStr;

use crate::ScopeType;

fn empty_str_as_none<'de, D>(deserializer: D) -> Result<Option<SmolStr>, D::Error>
where
	D: Deserializer<'de>,
{
	match Option::<String>::deserialize(deserializer)? {
		None => Ok(None),
		Some(s) if s.is_empty() => Ok(None),
		Some(s) => Ok(Some(s.into())),
	}
}



#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryAuthConfig {
	pub scope: ScopeType,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub token: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub auth: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub username: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub password: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub email: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub certfile: Option<SmolStr>,
	#[serde(
		default,
		deserialize_with = "empty_str_as_none",
		skip_serializing_if = "Option::is_none"
	)]
	pub keyfile: Option<SmolStr>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthConfig(Vec<RegistryAuthConfig>);

impl AuthConfig {
	pub fn for_scope(&self, scope: Option<&ScopeType>) -> Option<&RegistryAuthConfig> {
		let scope = scope.unwrap_or(&ScopeType::Npm);
		self.0.iter().rev().find(|config| config.scope == *scope)
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}
