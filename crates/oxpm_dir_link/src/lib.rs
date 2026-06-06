use std::io::Result;
use std::path::Path;

/// 目录链接管理
///
/// - Windows：junction
/// - Unix：symlink
pub struct DirLink;

impl DirLink {
    /// 创建目录链接。source 必须是目录
    pub fn create(source: &Path, dest: &Path) -> Result<()> {
        if !source.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("source is not a directory: {}", source.display()),
            ));
        }

        Self::remove(dest)?;

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        create_dir_link(source, dest)
    }

    /// 移除目录链接，不影响链接目标
    pub fn remove(path: &Path) -> Result<()> {
        if path.symlink_metadata().is_err() {
            return Ok(());
        }

        let _ = std::fs::remove_dir(path);
        Ok(())
    }
}

#[cfg(windows)]
fn create_dir_link(source: &Path, dest: &Path) -> Result<()> {
    junction::create(source, dest)?;
    Ok(())
}

#[cfg(not(windows))]
fn create_dir_link(source: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> std::path::PathBuf {
        let tmp = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&tmp);
        tmp
    }

    #[test]
    fn create_and_remove() {
        let tmp = test_dir("oxpm_dirlink_cr_test");
        let src = tmp.join("source");
        let dest = tmp.join("dest");

        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("file.txt"), "hello").unwrap();

        DirLink::create(&src, &dest).unwrap();
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(dest.join("file.txt")).unwrap(), "hello");

        DirLink::remove(&dest).unwrap();
        assert!(!dest.exists());
        assert!(src.exists());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_overwrites_existing() {
        let tmp = test_dir("oxpm_dirlink_overwrite_test");
        let src1 = tmp.join("v1");
        let src2 = tmp.join("v2");
        let dest = tmp.join("link");

        std::fs::create_dir_all(&src1).unwrap();
        std::fs::write(src1.join("ver"), "1").unwrap();
        std::fs::create_dir_all(&src2).unwrap();
        std::fs::write(src2.join("ver"), "2").unwrap();

        DirLink::create(&src1, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("ver")).unwrap(), "1");

        DirLink::create(&src2, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("ver")).unwrap(), "2");

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn create_nested_parent() {
        let tmp = test_dir("oxpm_dirlink_nested_test");
        let src = tmp.join("src");
        let dest = tmp.join("a/b/c/link");

        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("f"), "x").unwrap();

        DirLink::create(&src, &dest).unwrap();
        assert!(dest.exists());

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        let tmp = test_dir("oxpm_dirlink_noop_test");
        DirLink::remove(&tmp.join("nonexistent")).unwrap();
    }
}
