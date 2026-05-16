use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error("failed to read package.json")]
	Io(#[from] std::io::Error),
	#[error("failed to parse package.json")]
	Parse(#[from] serde_json::Error),
}