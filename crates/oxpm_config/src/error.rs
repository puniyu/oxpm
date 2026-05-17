use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("failed to read config file: {0}")]
	Io(#[from] std::io::Error),
	#[error("failed to parse toml config: {0}")]
	Toml(#[from] toml::de::Error),
}