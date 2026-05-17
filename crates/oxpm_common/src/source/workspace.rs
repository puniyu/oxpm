use smol_str::SmolStr;
use serde::{Deserialize, Serialize};
use sugar_path::SugarPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceSource {
	path: SmolStr,
}

impl WorkspaceSource {

	pub fn new(path: SmolStr) -> Self {
		Self { path }
	}


	pub fn path(&self) -> &str {
		self.path.as_str()
	}


	pub fn as_str(&self) -> &str {
		self.path.as_str()
	}


	pub fn is_absolute(&self) -> bool {
		self.path.as_str().as_path().is_absolute()
	}

	pub fn is_relative(&self) -> bool {
		!self.is_absolute()
	}

	pub fn normalize(&self) -> SmolStr {
		self.path.as_str().as_path().normalize().to_string_lossy().to_string().into()
	}

	pub fn to_absolute(&self, base: &str) -> SmolStr {
		self.path.as_str().absolutize_with(base).to_string_lossy().into()
	}
}

impl std::fmt::Display for WorkspaceSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}

impl AsRef<str> for WorkspaceSource {
	fn as_ref(&self) -> &str {
		self.path.as_str()
	}
}

impl From<&str> for WorkspaceSource {
	fn from(path: &str) -> Self {
		Self::new(SmolStr::new(path))
	}
}

impl From<String> for WorkspaceSource {
	fn from(path: String) -> Self {
		Self::new(SmolStr::new(path))
	}
}

impl From<WorkspaceSource> for SmolStr {
	fn from(source: WorkspaceSource) -> SmolStr {
		source.path
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_str() {
		let source = WorkspaceSource::from("packages/my-lib");
		assert_eq!(source.path(), "packages/my-lib");
	}

	#[test]
	fn display() {
		let source = WorkspaceSource::from("./apps/web");
		assert_eq!(format!("{}", source), "./apps/web");
	}
}