use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;
use tar::Archive;

use crate::Error;

/// 将 tarball 解压到目标目录，剥离 npm tarball 的 `package/` 前缀
pub fn extract_tarball(tarball_path: &Path, dest: &Path) -> std::result::Result<(), Error> {
    let file = fs::File::open(tarball_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    for entry in archive.entries().map_err(|e| Error::Archive(e.to_string()))? {
        let mut entry = entry.map_err(|e| Error::Archive(e.to_string()))?;
        let entry_path = entry
            .path()
            .map_err(|e| Error::Archive(e.to_string()))?;

        let stripped = entry_path
            .strip_prefix("package")
            .unwrap_or(&entry_path);

        if stripped.as_os_str().is_empty() {
            continue;
        }

        entry
            .unpack_in(dest)
            .map_err(|e| Error::Archive(e.to_string()))?;
    }

    Ok(())
}

/// 将目录内容复制到目标目录
pub fn copy_dir(source: &Path, dest: &Path) -> std::result::Result<(), Error> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let dest_path = dest.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }

    Ok(())
}

/// 从 integrity SRI 字符串计算 blake3 hash，用作 store 路径
pub fn integrity_hash(integrity: &str) -> String {
    blake3::hash(integrity.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_hash_deterministic() {
        let h1 = integrity_hash("sha512-abc123");
        let h2 = integrity_hash("sha512-abc123");
        assert_eq!(h1, h2);
    }

    #[test]
    fn integrity_hash_different_inputs() {
        let h1 = integrity_hash("sha512-abc123");
        let h2 = integrity_hash("sha512-def456");
        assert_ne!(h1, h2);
    }
}
