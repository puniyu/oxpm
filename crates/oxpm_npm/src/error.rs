use smol_str::SmolStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("package `{0}` not found in registry")]
    PackageNotFound(SmolStr),

    #[error("no matching version for `{name}` with range `{range}`")]
    NoMatchingVersion { name: SmolStr, range: SmolStr },

    #[error("invalid dist-tag `{tag}` for package `{name}`")]
    InvalidDistTag { name: SmolStr, tag: SmolStr },

    #[error("registry error: {0}")]
    Registry(#[from] oxpm_registry::Error),

    #[error("semver error: {0}")]
    Semver(#[from] oxpm_semver::Error),
}
