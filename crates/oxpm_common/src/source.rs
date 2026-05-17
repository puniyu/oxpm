mod file;
pub use file::FileSource;
mod git;
pub use git::GitSource;
mod link;
pub use link::LinkSource;
mod registry;
pub use registry::RegistrySource;
mod tarball;
pub use tarball::{TarballSource, TarballType};
mod workspace;
pub use workspace::WorkspaceSource;

use crate::SourceError;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SourceType {
	Registry(RegistrySource),
	Git(GitSource),
	File(FileSource),
	Link(LinkSource),
	Workspace(WorkspaceSource),
	Tarball(TarballSource),
}

impl SourceType {
	#[inline]
	pub fn parse(source: &str) -> Self {
		if let Some(stripped) = source.strip_prefix("git+") {
			return match GitSource::parse_git_url(stripped) {
				Some(s) => SourceType::Git(s),
				None => SourceType::Git(GitSource::try_from(stripped).expect("invalid git source")),
			};
		}
		if let Some(path) = source.strip_prefix("file:") {
			return SourceType::File(FileSource::from(path));
		}
		if let Some(path) = source.strip_prefix("link:") {
			return SourceType::Link(LinkSource::from(path));
		}
		if let Some(path) = source.strip_prefix("workspace:") {
			return SourceType::Workspace(WorkspaceSource::from(path));
		}
		if let Some(path) = source.strip_prefix("tarball:") {
			return SourceType::Tarball(TarballSource::from_path(path).expect("invalid tarball path"));
		}
		if let Some(origin) = source.strip_prefix("registry+") {
			return SourceType::Registry(RegistrySource::try_from(origin).expect("invalid registry source"));
		}
		SourceType::Registry(RegistrySource::try_from(source).expect("invalid source"))
	}

	#[inline]
	pub fn to_source_string(&self) -> SmolStr {
		match self {
			SourceType::Registry(source) => SmolStr::new(format!("registry+{}", source.as_str())),
			SourceType::Git(source) => SmolStr::new(format!("git+{}", source)),
			SourceType::File(source) => SmolStr::new(format!("file:{}", source.path())),
			SourceType::Link(source) => SmolStr::new(format!("link:{}", source.path())),
			SourceType::Workspace(source) => SmolStr::new(format!("workspace:{}", source.path())),
			SourceType::Tarball(source) => SmolStr::new(format!("tarball:{}", source.path())),
		}
	}

	#[inline]
	pub fn is_registry(&self) -> bool {
		matches!(self, SourceType::Registry(_))
	}

	#[inline]
	pub fn is_git(&self) -> bool {
		matches!(self, SourceType::Git(_))
	}

	#[inline]
	pub fn is_file(&self) -> bool {
		matches!(self, SourceType::File(_))
	}

	#[inline]
	pub fn is_link(&self) -> bool {
		matches!(self, SourceType::Link(_))
	}

	#[inline]
	pub fn is_workspace(&self) -> bool {
		matches!(self, SourceType::Workspace(_))
	}

	#[inline]
	pub fn is_tarball(&self) -> bool {
		matches!(self, SourceType::Tarball(_))
	}

	#[inline]
	pub fn is_local(&self) -> bool {
		matches!(
			self,
			SourceType::File(_)
				| SourceType::Link(_)
				| SourceType::Workspace(_)
				| SourceType::Tarball(_)
		)
	}

	#[inline]
	pub fn as_registry(&self) -> Option<&RegistrySource> {
		match self {
			SourceType::Registry(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_git(&self) -> Option<&GitSource> {
		match self {
			SourceType::Git(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_file(&self) -> Option<&FileSource> {
		match self {
			SourceType::File(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_link(&self) -> Option<&LinkSource> {
		match self {
			SourceType::Link(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_workspace(&self) -> Option<&WorkspaceSource> {
		match self {
			SourceType::Workspace(s) => Some(s),
			_ => None,
		}
	}

	#[inline]
	pub fn as_tarball(&self) -> Option<&TarballSource> {
		match self {
			SourceType::Tarball(s) => Some(s),
			_ => None,
		}
	}
}

impl std::fmt::Display for SourceType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_source_string())
	}
}

impl AsRef<str> for SourceType {
	fn as_ref(&self) -> &str {
		match self {
			SourceType::Registry(s) => s.as_str(),
			SourceType::Git(s) => s.as_str(),
			SourceType::File(s) => s.as_str(),
			SourceType::Link(s) => s.as_str(),
			SourceType::Workspace(s) => s.as_str(),
			SourceType::Tarball(s) => s.as_str(),
		}
	}
}

impl TryFrom<&str> for SourceType {
	type Error = SourceError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if value.is_empty() {
			return Err(SourceError("empty source".to_string()));
		}
		Ok(Self::parse(value))
	}
}

impl From<RegistrySource> for SourceType {
	fn from(source: RegistrySource) -> Self {
		Self::Registry(source)
	}
}

impl From<GitSource> for SourceType {
	fn from(source: GitSource) -> Self {
		Self::Git(source)
	}
}

impl From<FileSource> for SourceType {
	fn from(source: FileSource) -> Self {
		Self::File(source)
	}
}

impl From<LinkSource> for SourceType {
	fn from(source: LinkSource) -> Self {
		Self::Link(source)
	}
}

impl From<WorkspaceSource> for SourceType {
	fn from(source: WorkspaceSource) -> Self {
		Self::Workspace(source)
	}
}

impl From<TarballSource> for SourceType {
	fn from(source: TarballSource) -> Self {
		Self::Tarball(source)
	}
}

impl Default for SourceType {
	fn default() -> Self {
		Self::Registry(RegistrySource::try_from("https://registry.npmjs.org").unwrap())
	}
}
