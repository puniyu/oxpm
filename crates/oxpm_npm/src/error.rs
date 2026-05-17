use smol_str::SmolStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LockFileError {
	#[error("failed to read lockfile")]
	Io(#[from] std::io::Error),
	#[error("unsupported lockfileVersion: {0}")]
	UnsupportedLockfileVersion(u64),
	#[error("missing package version for package {name} at {path}")]
	MissingPackageVersion { path: SmolStr, name: SmolStr },
	#[error("invalid npm package path: {0}")]
	InvalidPackagePath(SmolStr),
	#[error("failed to parse lockfile")]
	Parse(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("lockfile error: {0}")]
    LockFile(#[from] LockFileError),
}
