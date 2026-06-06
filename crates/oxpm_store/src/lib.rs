use oxpm_path::OxpmStoreDir;
use oxpm_package_json::PackageJson;
use std::path::{Path, PathBuf};

mod error;
pub use error::Error;
pub mod tarball;

pub type Result<T> = std::result::Result<T, Error>;

pub struct OxpmStore {
    dir: OxpmStoreDir,
}

impl OxpmStore {
    pub fn new(dir: OxpmStoreDir) -> Self {
        Self { dir }
    }

    /// 从 tarball 安装包到 store，返回 store 中的路径
    pub fn install_from_tarball(&self, tarball_path: &Path, integrity: &str) -> Result<PathBuf> {
        self.install_with(integrity, |tmp| tarball::extract_tarball(tarball_path, tmp))
    }

    /// 从本地目录安装包到 store，返回 store 中的路径
    pub fn install_from_dir(&self, source_dir: &Path, integrity: &str) -> Result<PathBuf> {
        self.install_with(integrity, |tmp| tarball::copy_dir(source_dir, tmp))
    }

    /// 检查包是否已在 store 中
    pub fn has(&self, integrity: &str) -> bool {
        self.dir.exists(&tarball::integrity_hash(integrity))
    }

    /// 获取包在 store 中的路径
    pub fn get_path(&self, integrity: &str) -> Option<PathBuf> {
        let path = self.dir.package_path(&tarball::integrity_hash(integrity));
        path.exists().then_some(path)
    }

    /// 从 store 中读取包的 package.json
    pub fn read_package_json(&self, integrity: &str) -> Result<PackageJson> {
        let path = self
            .get_path(integrity)
            .ok_or_else(|| Error::NotInStore(integrity.to_string()))?;

        Ok(PackageJson::load_from_path(path.join("package.json"))?)
    }

    fn install_with(
        &self,
        integrity: &str,
        populate: impl FnOnce(&Path) -> std::result::Result<(), Error>,
    ) -> Result<PathBuf> {
        let hash = tarball::integrity_hash(integrity);
        let dest = self.dir.package_path(&hash);

        if dest.exists() {
            return Ok(dest);
        }

        let tmp = self.dir.path().join(format!(".tmp-{hash}"));
        let _ = fs::remove_dir_all(&tmp);

        populate(&tmp)?;

        match fs::rename(&tmp, &dest) {
            Ok(()) => Ok(dest),
            Err(_) => {
                tarball::copy_dir(&tmp, &dest)?;
                let _ = fs::remove_dir_all(&tmp);
                Ok(dest)
            }
        }
    }
}

use std::fs;

#[cfg(test)]
mod tests {
    use super::*;
    use oxpm_path::OxpmDir;

    fn test_store(name: &str) -> (OxpmStore, PathBuf) {
        let tmp = std::env::temp_dir().join(format!("oxpm_store_{name}"));
        let _ = fs::remove_dir_all(&tmp);
        let dir = OxpmDir::from_path(&tmp);
        dir.ensure_dirs().unwrap();
        let store = OxpmStore::new(dir.store());
        (store, tmp)
    }

    #[test]
    fn has_returns_false_for_missing() {
        let (store, tmp) = test_store("has_false");
        assert!(!store.has("sha512-nonexistent"));
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn install_from_dir_creates_entry() {
        let (store, tmp) = test_store("creates_entry");

        let src = tmp.join("src-pkg");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("package.json"), r#"{"name":"test","version":"1.0.0"}"#).unwrap();
        fs::write(src.join("index.js"), "console.log('hello')").unwrap();

        let integrity = "sha512-test-integrity";
        let path = store.install_from_dir(&src, integrity).unwrap();

        assert!(path.exists());
        assert!(path.join("package.json").exists());
        assert!(path.join("index.js").exists());
        assert!(store.has(integrity));
        assert_eq!(store.get_path(integrity), Some(path));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn install_from_dir_is_idempotent() {
        let (store, tmp) = test_store("idempotent");

        let src = tmp.join("src-pkg2");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("package.json"), r#"{"name":"test","version":"1.0.0"}"#).unwrap();

        let integrity = "sha512-idempotent-test";
        let p1 = store.install_from_dir(&src, integrity).unwrap();
        let p2 = store.install_from_dir(&src, integrity).unwrap();
        assert_eq!(p1, p2);

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn read_package_json_works() {
        let (store, tmp) = test_store("read_pkg");

        let src = tmp.join("src-pkg3");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            src.join("package.json"),
            r#"{"name":"my-pkg","version":"2.0.0"}"#,
        )
        .unwrap();

        let integrity = "sha512-read-test";
        store.install_from_dir(&src, integrity).unwrap();

        let pkg = store.read_package_json(integrity).unwrap();
        assert_eq!(pkg.name.unwrap().as_str(), "my-pkg");
        assert_eq!(pkg.version.unwrap().to_string(), "2.0.0");

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn get_path_returns_none_for_missing() {
        let (store, tmp) = test_store("get_none");
        assert!(store.get_path("sha512-missing").is_none());
        fs::remove_dir_all(&tmp).ok();
    }
}
