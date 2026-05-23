use async_trait::async_trait;
use flate2::read::GzDecoder;
use oxpm_dep::{DepNode, DepType};
use oxpm_package_json::PackageJson;
use oxpm_resolver::Resolver;
use smol_str::SmolStr;
use std::io::Read;
use std::path::PathBuf;
use tar::Archive;

pub struct TarballResolver;

#[async_trait]
impl Resolver for TarballResolver {
    type Source = super::TarballSource;

    async fn resolve(
        &self,
        name: &SmolStr,
        range: &str,
        dep_type: DepType,
    ) -> std::result::Result<DepNode, Box<dyn std::error::Error + Send + Sync>> {
        let tarball_path = PathBuf::from(range);
        let json = tokio::task::spawn_blocking({
            let tarball_path = tarball_path.clone();
            move || read_file_from_tarball(&tarball_path, "package.json")
        })
        .await
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Interrupted, "spawn cancelled"))??;

        let pkg = PackageJson::load_from_str(&json)?;
        let version = pkg.version.clone().ok_or_else(|| super::Error::VersionNotFound {
            name: name.clone(),
            range: SmolStr::new(range),
        })?;

        let source_str = SmolStr::new(format!("tarball:{}", range));
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

fn read_file_from_tarball(tarball_path: &PathBuf, file_path: &str) -> std::io::Result<String> {
    let reader = std::fs::File::open(tarball_path)?;
    let reader = std::io::BufReader::new(reader);
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;
        let entry_str = entry_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid path in tarball")
        })?;
        if entry_str == file_path || entry_str == format!("package/{}", file_path) {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            return Ok(contents);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("file not found in tarball: {}", file_path),
    ))
}