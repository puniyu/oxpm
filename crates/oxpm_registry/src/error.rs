use thiserror::Error;
use smol_str::SmolStr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse registry response")]
    Parse(#[from] serde_json::Error),
    #[error("version `{version}` not found for package `{name}`")]
    VersionNotFound { name: SmolStr, version: SmolStr },
    #[error("package `{0}` not found")]
    PackageNotFound(SmolStr),
}