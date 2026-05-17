use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use flate2::read::GzDecoder;
use tar::Archive;
use tokio::fs;
use url::Url;

use crate::error::Error;
use crate::Result;

// oxpm 在系统临时目录下的缓存根目录
static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| std::env::temp_dir().join("oxpm"));

/// 下载 tarball 并解压到临时目录
pub async fn download_and_extract(tarball_url: &Url) -> Result<PathBuf> {
    let hash = blake3::hash(tarball_url.as_str().as_bytes());
    let cache_subdir = CACHE_DIR.join(format!("pkg-{}", &hash.to_hex().to_string()[..16]));
    let tarball_path = cache_subdir.join("package.tgz");
    let extracted_dir = cache_subdir.join("extracted");

    if extracted_dir.join("package.json").exists() {
        return Ok(extracted_dir);
    }

    fs::create_dir_all(&cache_subdir).await?;
    let response = reqwest::get(tarball_url.as_str()).await?;
    let bytes = response.bytes().await?;
    fs::write(&tarball_path, &bytes).await?;

    extract_tarball_internal(&tarball_path, &extracted_dir).await?;

    Ok(extracted_dir)
}

/// 解压本地 tarball 文件到临时目录
pub async fn extract_tarball(tarball_path: &str) -> Result<PathBuf> {
    let hash = blake3::hash(tarball_path.as_bytes());
    let cache_subdir = CACHE_DIR.join(format!("pkg-{}", &hash.to_hex().to_string()[..16]));
    let extracted_dir = cache_subdir.join("extracted");

    if extracted_dir.join("package.json").exists() {
        return Ok(extracted_dir);
    }

    fs::create_dir_all(&cache_subdir).await?;
    let path = PathBuf::from(tarball_path);
    extract_tarball_internal(&path, &extracted_dir).await?;

    Ok(extracted_dir)
}

/// 解压 tarball 到目标目录
pub async fn extract_tarball_internal(tarball_path: &Path, dest: &Path) -> Result<()> {
    let bytes = tokio::fs::read(tarball_path).await?;
    let decoder = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest).map_err(Error::Io)?;
    Ok(())
}