use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid source format: {0}")]
pub struct SourceError(pub String);

#[derive(Debug, Error)]
pub enum Error {
	#[error("source error: {0}")]
	Source(#[from] SourceError)
}