use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("path error: {0}")]
    Path(#[from] oxpm_path::Error),

    #[error("package.json error: {0}")]
    PackageJson(#[from] oxpm_package_json::Error),

    #[error("integrity `{0}` is not in store")]
    NotInStore(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("archive error: {0}")]
    Archive(String),
}
