use async_trait::async_trait;
use oxpm_dep::{DepNode, DepType};
use oxpm_registry::Registry;
use oxpm_resolver::Resolver;
use oxpm_semver::{Version, VersionRange};
use smol_str::SmolStr;
use url::Url;

use crate::Error;

pub struct NpmResolver {
    registry: Registry,
    registry_url: Url,
}

impl NpmResolver {
    pub fn new(registry: Registry, registry_url: Url) -> Self {
        Self { registry, registry_url }
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn registry_url(&self) -> &Url {
        &self.registry_url
    }
}

#[async_trait]
impl Resolver for NpmResolver {
    type Source = super::NpmSource;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>> {
        let package = self.registry.package(name).await?;

        let version_str = if range == "latest" || range.is_empty() {
            package
                .dist_tags
                .get("latest")
                .ok_or_else(|| Error::InvalidDistTag {
                    name: name.clone(),
                    tag: SmolStr::new("latest"),
                })?
                .clone()
        } else if !range.contains('.') && !range.contains('^') && !range.contains('~')
            && !range.contains('>')
            && !range.contains('<')
            && !range.contains('=')
            && !range.contains('*')
            && !range.contains('|')
        {
            package
                .dist_tags
                .get(range)
                .ok_or_else(|| Error::InvalidDistTag {
                    name: name.clone(),
                    tag: SmolStr::new(range),
                })?
                .clone()
        } else {
            let version_range = VersionRange::parse(range)?;

            let mut best: Option<(&SmolStr, &Version)> = None;
            for (ver_str, pkg_version) in &package.versions {
                if version_range.matches(&pkg_version.version) {
                    match &best {
                        Some((_, best_ver)) => {
                            if pkg_version.version > **best_ver {
                                best = Some((ver_str, &pkg_version.version));
                            }
                        }
                        None => best = Some((ver_str, &pkg_version.version)),
                    }
                }
            }

            let (ver_str, _) = best.ok_or_else(|| Error::NoMatchingVersion {
                name: name.clone(),
                range: SmolStr::new(range),
            })?;

            ver_str.clone()
        };

        let pkg_version = package.versions.get(version_str.as_str()).unwrap();
        let version = pkg_version.version.clone();

        let base = self.registry_url.as_str().trim_end_matches('/');
        let source_str = SmolStr::new(format!("registry+{base}"));

        let mut node = DepNode::new(name.clone(), version, dep_type, source_str)
            .with_range(SmolStr::new(range));

        if let Some(integrity) = &pkg_version.dist.integrity {
            node = node.with_integrity(integrity.clone());
        }

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_creation() {
        let url = Url::parse("https://registry.npmjs.org/").unwrap();
        let registry = Registry::new(url.clone()).unwrap();
        let resolver = NpmResolver::new(registry, url.clone());
        assert_eq!(resolver.registry_url().as_str(), "https://registry.npmjs.org/");
    }
}
