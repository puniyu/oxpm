use std::convert::TryFrom;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use url::Url;

use crate::error::SourceError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitSource {
	git_url: Url,
	owner: SmolStr,
	repo: SmolStr,
	ref_: Option<SmolStr>,
}

impl GitSource {
	pub fn new(url: Url, owner: SmolStr, repo: SmolStr, ref_: Option<SmolStr>) -> Self {
		Self { git_url: url, owner, repo, ref_ }
	}

	pub fn url(&self) -> &Url {
		&self.git_url
	}

	pub fn owner(&self) -> &str {
		self.owner.as_str()
	}

	pub fn repo(&self) -> &str {
		self.repo.as_str()
	}

	pub fn reference(&self) -> Option<&str> {
		self.ref_.as_deref()
	}

	pub fn is_commit(&self) -> bool {
		self.ref_.as_ref().is_some_and(|r| r.len() >= 7 && !r.contains('#') && !r.contains('/'))
	}

	#[inline]
	pub fn as_str(&self) -> &str {
		self.git_url.as_str().trim_end_matches('/')
	}
}

impl std::fmt::Display for GitSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.ref_ {
			Some(r) => write!(f, "{}#{}", self.git_url, r),
			None => write!(f, "{}", self.git_url),
		}
	}
}

impl AsRef<str> for GitSource {
	fn as_ref(&self) -> &str {
		self.git_url.as_str()
	}
}

impl TryFrom<&str> for GitSource {
	type Error = SourceError;

	fn try_from(url: &str) -> Result<Self, Self::Error> {
		Self::parse_git_url(url).ok_or_else(|| SourceError(url.to_string()))
	}
}

impl GitSource {
	pub fn parse_git_url(input: &str) -> Option<Self> {
		let (url_part, ref_part) = input.split_once('#').unwrap_or((input, ""));

		let url_str = url_part.trim_end_matches('/');

		let url = Url::parse(url_str).ok()?;
		let parts: Vec<&str> = url.path().split('/').collect();
		if parts.len() < 2 {
			return None;
		}

		let repo = parts.last()?.strip_suffix(".git").unwrap_or(*parts.last()?);
		let owner = parts.get(parts.len() - 2)?;

		Some(Self {
			git_url: url.clone(),
			owner: SmolStr::new(owner),
			repo: SmolStr::new(repo),
			ref_: if ref_part.is_empty() { None } else { Some(SmolStr::new(ref_part)) },
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_github_url() {
		let source = GitSource::parse_git_url("git+https://github.com/user/repo#abc123").unwrap();
		assert_eq!(source.owner(), "user");
		assert_eq!(source.repo(), "repo");
		assert_eq!(source.reference(), Some("abc123"));
	}

	#[test]
	fn parse_url_without_ref() {
		let source = GitSource::parse_git_url("git+https://github.com/user/repo").unwrap();
		assert_eq!(source.owner(), "user");
		assert_eq!(source.repo(), "repo");
		assert!(source.reference().is_none());
	}

	#[test]
	fn display_with_ref() {
		let source = GitSource::new(
			Url::parse("https://github.com/user/repo").unwrap(),
			SmolStr::new("user"),
			SmolStr::new("repo"),
			Some(SmolStr::new("main")),
		);
		assert_eq!(format!("{}", source), "https://github.com/user/repo#main");
	}
}