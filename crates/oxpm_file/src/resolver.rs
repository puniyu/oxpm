use async_trait::async_trait;
use oxpm_dep::{DepNode, DepType};
use oxpm_package_json::PackageJson;
use oxpm_resolver::Resolver;
use smol_str::SmolStr;
use std::path::PathBuf;

pub struct FileResolver;

#[async_trait]
impl Resolver for FileResolver {
    type Source = super::FileSource;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>> {
        let path = range;
        let package_json_path = PathBuf::from(path).join("package.json");
        let pkg = PackageJson::load_from_path(&package_json_path)?;
        let version = pkg.version.clone().ok_or_else(|| super::Error::VersionNotFound {
            name: name.clone(),
            range: SmolStr::new(range),
        })?;

        let source_str = SmolStr::new(format!("file:{}", path));
        let node = DepNode::new(
            name.clone(),
            version,
            dep_type,
            source_str,
        )
        .with_range(SmolStr::new(range));

        Ok(node)
    }
}