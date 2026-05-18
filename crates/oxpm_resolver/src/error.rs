use thiserror::Error;
use smol_str::SmolStr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("package not found: {0}")]
    PackageNotFound(SmolStr),

    #[error("version not found for {name}: {range}")]
    VersionNotFound { name: SmolStr, range: SmolStr },

    #[error("cycle detected: {name} in path {path:?}")]
    CycleDetected { name: SmolStr, path: Vec<SmolStr> },

    #[error("version conflict: {name}")]
    VersionConflict { name: SmolStr },

    #[error("source error: {0}")]
    Source(#[from] oxpm_common::SourceError),

    #[error("registry error: {0}")]
    Registry(#[from] oxpm_registry::Error),

    #[error("semver error: {0}")]
    Semver(#[from] oxpm_semver::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("packageJson error: {0}")]
    PackageJson(#[from] oxpm_package_json::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("tar error: {0}")]
    Tar(#[from] oxpm_tar::Error),
}