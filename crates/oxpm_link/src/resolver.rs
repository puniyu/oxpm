use async_trait::async_trait;
use oxpm_dep::{DepNode, DepType};
use oxpm_package_json::PackageJson;
use oxpm_resolver::Resolver;
use smol_str::SmolStr;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct LinkResolver;

#[async_trait]
impl Resolver for LinkResolver {
    type Source = super::LinkSource;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>> {
        let link_target = resolve_link_target(PathBuf::from(range))
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        let package_json_path = link_target.join("package.json");
        let pkg = PackageJson::load_from_path(&package_json_path)?;
        let version = pkg.version.clone().ok_or_else(|| super::Error::VersionNotFound {
            name: name.clone(),
            range: SmolStr::new(range),
        })?;

        let source_str = SmolStr::new(format!("link:{}", range));
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

fn resolve_link_target(mut path: PathBuf) -> std::io::Result<PathBuf> {
    let mut visited = HashSet::new();
    loop {
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            if !visited.insert(path.clone()) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "symlink cycle detected",
                ));
            }
            let target = std::fs::read_link(&path)?;
            path = if target.is_absolute() {
                target
            } else {
                path.parent()
                    .map(|p| p.join(&target))
                    .unwrap_or(target)
            };
        } else {
            return Ok(path);
        }
    }
}