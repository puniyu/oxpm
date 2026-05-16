use std::{collections::HashMap, sync::Arc};

use smol_str::SmolStr;

use ini::Ini;
pub type Result<T> = std::result::Result<T, ini::Error>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RegistryConfig {
	pub auth: Option<SmolStr>,
	pub auth_token: Option<SmolStr>,
	pub username: Option<SmolStr>,
	pub password: Option<SmolStr>,
	pub email: Option<SmolStr>,
	pub certfile: Option<SmolStr>,
	pub keyfile: Option<SmolStr>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NpmRc {
	pub registry: Option<SmolStr>,
	pub scope_registries: HashMap<String, String>,
	pub registry_configs: HashMap<String, Arc<RegistryConfig>>,
}

impl NpmRc {
	pub fn load_from_path(path: impl AsRef<std::path::Path>) -> Result<Self> {
		Ok(Self::from_ini(Ini::load_from_file(path)?))
	}

	pub fn load_from_str(content: &str) -> Result<Self> {
		Ok(Self::from_ini(
			Ini::load_from_str(content).map_err(ini::Error::Parse)?,
		))
	}

	fn from_ini(ini: Ini) -> Self {
		let mut rc = NpmRc::default();

		for (sec, prop) in ini.iter() {
			let sec_name = sec.unwrap_or_default();
			let sec_name_lower = sec_name.to_lowercase();

			if let Some(r) = prop.get("registry") {
				if sec_name_lower == "registry" {
					rc.registry = Some(SmolStr::from(r));
				} else if let Some(scope) = sec_name.strip_prefix("@") {
					rc.scope_registries.insert(scope.to_string(), r.to_string());
				}
			}

			if let Some(url) = sec_name.strip_prefix("//") {
				let url = url.to_string();
				let config = RegistryConfig {
					auth: prop.get("authToken").map(SmolStr::from),
					auth_token: prop.get("_authToken").map(SmolStr::from),
					username: prop.get("username").map(SmolStr::from),
					password: prop.get("password").map(SmolStr::from),
					email: prop.get("email").map(SmolStr::from),
					certfile: prop.get("certfile").map(SmolStr::from),
					keyfile: prop.get("keyfile").map(SmolStr::from),
				};
				rc.registry_configs.insert(url, Arc::new(config));
			}
		}

		rc
	}
}