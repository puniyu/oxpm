use thiserror::Error;
use smol_str::SmolStr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("package not found: {0}")]
    PackageNotFound(SmolStr),

    #[error("version not found for {name}: {range}")]
    VersionNotFound { name: SmolStr, range: SmolStr },

    #[error("source error: {0}")]
    Source(String),

    #[error("packageJson error: {0}")]
    PackageJson(#[from] oxpm_package_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("symlink cycle detected")]
    SymlinkCycle,
}