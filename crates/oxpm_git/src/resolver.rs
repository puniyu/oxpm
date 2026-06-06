use async_trait::async_trait;
use git2::{build::CheckoutBuilder, FetchOptions, Repository};
use oxpm_dep::{DepNode, DepType};
use oxpm_package_json::PackageJson;
use oxpm_resolver::Resolver;
use smol_str::SmolStr;
use std::path::{Path, PathBuf};

pub struct GitResolver {
    cache_dir: PathBuf,
}

impl GitResolver {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    fn repo_cache_path(&self, url: &str) -> PathBuf {
        let hash = blake3::hash(url.as_bytes()).to_hex().to_string();
        self.cache_dir.join(&hash[..12])
    }
}

#[async_trait]
impl Resolver for GitResolver {
    type Source = super::GitSource;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>> {
        let url = range.strip_prefix("git+").unwrap_or(range);
        let git_source = super::GitSource::parse_git_url(url)
            .ok_or_else(|| super::Error::Source(url.to_string()))?;

        let git_url = git_source.url().to_string();
        let ref_ = git_source.reference().map(|s| s.to_string());
        let cache_path = self.repo_cache_path(&git_url);

        let workdir = tokio::task::spawn_blocking(move || {
            clone_or_update(&git_url, ref_.as_deref(), &cache_path)
        })
        .await
        .map_err(|e| super::Error::Source(format!("task join error: {e}")))??;

        let pkg_path = workdir.join("package.json");
        let pkg = PackageJson::load_from_path(&pkg_path)?;
        let version = pkg.version.ok_or_else(|| super::Error::VersionNotFound {
            name: name.clone(),
            range: SmolStr::new(range),
        })?;

        let source_str = match git_source.reference() {
            Some(_) => SmolStr::new(format!("git+{url}")),
            None => SmolStr::new(format!("git+{url}")),
        };

        let node = DepNode::new(name.clone(), version, dep_type, source_str)
            .with_range(SmolStr::new(range));

        Ok(node)
    }
}

fn clone_or_update(
    url: &str,
    ref_: Option<&str>,
    cache_path: &Path,
) -> std::result::Result<PathBuf, super::Error> {
    let repo = if cache_path.join(".git").exists() {
        let repo = Repository::open(cache_path)?;
        let mut fetch_options = FetchOptions::new();
        repo.find_remote("origin")?.fetch(
            &[ref_.unwrap_or("HEAD")],
            Some(&mut fetch_options),
            None,
        )?;
        repo
    } else {
        Repository::clone(url, cache_path)?
    };

    checkout_ref(&repo, ref_)?;

    Ok(cache_path.to_path_buf())
}

fn checkout_ref(
    repo: &Repository,
    ref_: Option<&str>,
) -> std::result::Result<(), super::Error> {
    let ref_str = ref_.unwrap_or("HEAD");
    let obj = repo.revparse_single(ref_str)?;

    let mut checkout = CheckoutBuilder::new();
    checkout.force();
    repo.checkout_tree(&obj, Some(&mut checkout))?;
    if let Ok(reference) = repo.find_reference(&format!("refs/remotes/origin/{ref_str}")) {
        repo.set_head(reference.name().unwrap_or(&format!("refs/remotes/origin/{ref_str}")))?;
    } else if let Ok(reference) = repo.find_reference(&format!("refs/heads/{ref_str}")) {
        repo.set_head(reference.name().unwrap_or(&format!("refs/heads/{ref_str}")))?;
    } else {
        repo.set_head_detached(obj.id())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_creation() {
        let resolver = GitResolver::new(PathBuf::from("/tmp/oxpm-git-cache"));
        assert_eq!(resolver.cache_dir, PathBuf::from("/tmp/oxpm-git-cache"));
    }

    #[test]
    fn cache_path_consistent() {
        let resolver = GitResolver::new(PathBuf::from("/tmp/cache"));
        let p1 = resolver.repo_cache_path("https://github.com/user/repo");
        let p2 = resolver.repo_cache_path("https://github.com/user/repo");
        assert_eq!(p1, p2);
    }

    #[test]
    fn cache_path_different_urls() {
        let resolver = GitResolver::new(PathBuf::from("/tmp/cache"));
        let p1 = resolver.repo_cache_path("https://github.com/user/repo-a");
        let p2 = resolver.repo_cache_path("https://github.com/user/repo-b");
        assert_ne!(p1, p2);
    }
}
