use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use sugar_path::SugarPath;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileSource {
	path: SmolStr,
}

impl FileSource {
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
		self.path.as_str().as_path().normalize().to_slash_lossy().into_owned().into()
	}

	pub fn to_absolute(&self, base: &str) -> SmolStr {
		self.path.as_str().absolutize_with(base).to_slash_lossy().into_owned().into()
	}
}

impl std::fmt::Display for FileSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.path)
	}
}

impl AsRef<str> for FileSource {
	fn as_ref(&self) -> &str {
		self.path.as_str()
	}
}

impl From<&str> for FileSource {
	fn from(path: &str) -> Self {
		Self::new(SmolStr::new(path))
	}
}

impl From<String> for FileSource {
	fn from(path: String) -> Self {
		Self::new(SmolStr::new(path))
	}
}

impl From<FileSource> for SmolStr {
	fn from(source: FileSource) -> SmolStr {
		source.path
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_str() {
		let source = FileSource::from("../packages/local");
		assert_eq!(source.path(), "../packages/local");
	}

	#[test]
	fn display() {
		let source = FileSource::from("./dist");
		assert_eq!(format!("{}", source), "./dist");
	}

	#[test]
	fn is_relative() {
		let source = FileSource::from("../local");
		assert!(source.is_relative());
	}

	#[test]
	fn normalize() {
		let source = FileSource::from("./a/b/../c");
		assert_eq!(source.normalize().as_str(), "a/c");
	}
}
