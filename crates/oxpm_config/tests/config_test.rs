//! oxpm_config 集成测试

use oxpm_config::{Config, ScopeType};

/// 测试空配置加载
#[test]
fn test_load_empty_config() {
	let config = Config::load_from_str("").unwrap();
	assert!(!config.registry.is_empty());
	assert!(config.auth.is_empty());
	assert_eq!(config.registry.for_scope(None).as_str(), "https://registry.npmjs.org/");
	assert_eq!(config.registry.for_scope(Some(&ScopeType::Jsr)).as_str(), "https://npm.jsr.io/");
}


#[test]
fn test_parse_registry_scope() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "@puniyu"
url = "https://npm.example.com/"
"#,
	)
	.unwrap();

	let registry = config.registry.for_scope(Some(&ScopeType::from("@puniyu")));
	assert_eq!(registry.as_str(), "https://npm.example.com/");
}

#[test]
fn test_registry_falls_back_to_builtin_defaults() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "@puniyu"
url = "https://npm.example.com/"
"#,
	)
	.unwrap();

	assert_eq!(
		config.registry.for_scope(Some(&ScopeType::from("@unknown"))).as_str(),
		"https://registry.npmjs.org/"
	);
	assert_eq!(config.registry.for_scope(None).as_str(), "https://registry.npmjs.org/");
	assert_eq!(config.registry.for_scope(Some(&ScopeType::Jsr)).as_str(), "https://npm.jsr.io/");
}


#[test]
fn test_parse_auth_scope() {
	let config = Config::load_from_str(
		r#"
[[auth]]
scope = "@puniyu"
token = "secret"
username = "alice"
"#,
	)
	.unwrap();

	let auth = config.auth.for_scope(Some(&ScopeType::from("@puniyu"))).unwrap();
	assert_eq!(auth.token.as_deref(), Some("secret"));
	assert_eq!(auth.username.as_deref(), Some("alice"));
}


#[test]
fn test_parse_empty_auth_strings_as_none() {
	let config = Config::load_from_str(
		r#"
[[auth]]
scope = "@puniyu"
token = ""
username = ""
password = "secret"
"#,
	)
	.unwrap();

	let auth = config.auth.for_scope(Some(&ScopeType::from("@puniyu"))).unwrap();
	assert_eq!(auth.token, None);
	assert_eq!(auth.username, None);
	assert_eq!(auth.password.as_deref(), Some("secret"));
}


#[test]
fn test_auth_falls_back_to_none() {
	let config = Config::load_from_str(
		r#"
[[auth]]
scope = "@puniyu"
token = "secret"
"#,
	)
	.unwrap();

	assert!(config.auth.for_scope(Some(&ScopeType::from("@unknown"))).is_none());
	assert!(config.auth.for_scope(None).is_none());
}

#[test]
fn test_last_matching_scope_wins() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "@puniyu"
url = "https://first.example.com/"

[[registry]]
scope = "@puniyu"
url = "https://second.example.com/"

[[auth]]
scope = "@puniyu"
token = "first"

[[auth]]
scope = "@puniyu"
token = "second"
"#,
	)
	.unwrap();

	assert_eq!(
		config.registry.for_scope(Some(&ScopeType::from("@puniyu"))).as_str(),
		"https://second.example.com/"
	);
	assert_eq!(
		config.auth.for_scope(Some(&ScopeType::from("@puniyu"))).unwrap().token.as_deref(),
		Some("second")
	);
}


#[test]
fn test_serialize_skips_empty_auth_strings() {
	let config = Config::load_from_str(
		r#"
[[auth]]
scope = "@puniyu"
token = ""
auth = ""
username = "alice"
email = ""
certfile = ""
keyfile = "/tmp/key.pem"
"#,
	)
	.unwrap();

	let serialized = toml::to_string(&config).expect("serialize");
	assert!(!serialized.contains("token ="));
	assert!(!serialized.contains("auth ="));
	assert!(!serialized.contains("email ="));
	assert!(!serialized.contains("certfile ="));
	assert!(serialized.contains("username = \"alice\""));
	assert!(serialized.contains("keyfile = \"/tmp/key.pem\""));
}


#[test]
fn test_config_with_only_auth() {
	let config = Config::load_from_str(
		r#"
[[auth]]
scope = "@puniyu"
token = "secret"
"#,
	)
	.unwrap();

	assert!(!config.registry.is_empty());
	assert_eq!(config.registry.for_scope(None).as_str(), "https://registry.npmjs.org/");
	assert!(!config.auth.is_empty());
}

#[test]
fn test_config_with_only_registry() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "@puniyu"
url = "https://custom.example.com/"
"#,
	)
	.unwrap();

	assert!(!config.registry.is_empty());
	assert_eq!(
		config.registry.for_scope(Some(&ScopeType::from("@puniyu"))).as_str(),
		"https://custom.example.com/"
	);
	assert!(config.auth.is_empty());
}


#[test]
fn test_full_config() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "npm"
url = "https://registry.npmmirror.com/"

[[registry]]
scope = "@puniyu"
url = "https://npm.example.com/"

[[auth]]
scope = "npm"
token = "npm_token"

[[auth]]
scope = "@puniyu"
username = "user"
password = "pass"
"#,
	)
	.unwrap();

	// npm scope 使用自定义 registry
	assert_eq!(config.registry.for_scope(None).as_str(), "https://registry.npmmirror.com/");
	// @puniyu scope 使用自定义 registry
	assert_eq!(
		config.registry.for_scope(Some(&ScopeType::from("@puniyu"))).as_str(),
		"https://npm.example.com/"
	);
	// jsr 使用默认 registry
	assert_eq!(config.registry.for_scope(Some(&ScopeType::Jsr)).as_str(), "https://npm.jsr.io/");

	// npm auth
	let npm_auth = config.auth.for_scope(None).unwrap();
	assert_eq!(npm_auth.token.as_deref(), Some("npm_token"));

	// @puniyu auth
	let puniyu_auth = config.auth.for_scope(Some(&ScopeType::from("@puniyu"))).unwrap();
	assert_eq!(puniyu_auth.username.as_deref(), Some("user"));
	assert_eq!(puniyu_auth.password.as_deref(), Some("pass"));
}


#[test]
fn test_registry_config_serialize() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "@puniyu"
url = "https://custom.example.com/"
"#,
	)
	.unwrap();

	let serialized = toml::to_string(&config).unwrap();
	assert!(serialized.contains("[[registry]]"));
	assert!(serialized.contains("scope = \"@puniyu\""));
	assert!(serialized.contains("url = \"https://custom.example.com/\""));
}


#[test]
fn test_config_with_jsr_scope() {
	let config = Config::load_from_str(
		r#"
[[registry]]
scope = "jsr"
url = "https://custom.jsr.io/"
"#,
	)
	.unwrap();

	assert_eq!(config.registry.for_scope(Some(&ScopeType::Jsr)).as_str(), "https://custom.jsr.io/");
}
