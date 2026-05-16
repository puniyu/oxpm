use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read lockfile")]
    Io(#[from] std::io::Error),
    #[error("failed to parse lockfile")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize lockfile")]
    Serialize(#[from] toml::ser::Error),
}
