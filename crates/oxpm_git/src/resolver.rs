use async_trait::async_trait;
use oxpm_dep::{DepNode, DepType};
use oxpm_resolver::Resolver;
use smol_str::SmolStr;

pub struct GitResolver;

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
        let _git_source = super::GitSource::parse_git_url(url)
            .ok_or_else(|| Box::new(super::Error::Source(url.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;

        let source_str = SmolStr::new(range);
        let node = DepNode::new(
            name.clone(),
            oxpm_semver::Version::new(0, 0, 0),
            dep_type,
            source_str,
        )
        .with_range(SmolStr::new(range));

        Ok(node)
    }
}