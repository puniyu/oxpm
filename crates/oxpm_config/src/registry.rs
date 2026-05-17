use serde::{Deserialize, Serialize};
use url::Url;

use crate::ScopeType;

/// 默认 npm 注册表
pub const DEFAULT_NPM_REGISTRY: &str = "https://registry.npmjs.org/";
/// 默认 jsr 注册表
pub const DEFAULT_JSR_REGISTRY: &str = "https://npm.jsr.io/";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryScopeConfig {
	pub scope: ScopeType,
	pub url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegistryConfig(Vec<RegistryScopeConfig>);

impl Default for RegistryConfig {
	fn default() -> Self {
		let npm = RegistryScopeConfig {
			scope: ScopeType::Npm,
			url: Url::parse(DEFAULT_NPM_REGISTRY).expect("default registry url is valid"),
		};
		Self(vec![npm])
	}
}

impl RegistryConfig {
	/// 获取指定 scope 的注册表 URL
	pub fn for_scope(&self, scope: Option<&ScopeType>) -> Url {
		self.config_for_scope(scope)
			.map(|config| config.url.clone())
			.unwrap_or_else(|| default_registry_url(scope))
	}

	/// 获取指定 scope 的配置
	pub fn config_for_scope(&self, scope: Option<&ScopeType>) -> Option<&RegistryScopeConfig> {
		let scope = scope.unwrap_or(&ScopeType::Npm);
		self.0.iter().rev().find(|config| config.scope == *scope)
	}

	/// 是否有显式配置
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

/// 获取指定 scope 的默认注册表 URL
fn default_registry_url(scope: Option<&ScopeType>) -> Url {
	let url = match scope.unwrap_or(&ScopeType::Npm) {
		ScopeType::Jsr => DEFAULT_JSR_REGISTRY,
		_ => DEFAULT_NPM_REGISTRY,
	};
	Url::parse(url).expect("default registry url is valid")
}