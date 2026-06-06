use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("OXPM_HOME environment variable is not valid unicode")]
    InvalidHomeEnv,

    #[error("could not determine home directory")]
    HomeNotFound,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
