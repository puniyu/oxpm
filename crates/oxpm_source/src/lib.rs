use smol_str::SmolStr;

pub trait Source: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn as_str(&self) -> SmolStr;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("source error: {0}")]
    Source(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;