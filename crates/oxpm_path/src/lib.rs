use std::path::{Path, PathBuf};

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

const STORE_VERSION: &str = "v1";

/// `~/.oxpm` 根目录管理
#[derive(Debug, Clone)]
pub struct OxpmDir {
    path: PathBuf,
}

impl OxpmDir {
    /// 自动检测：优先 `$OXPM_HOME`，否则 `~/.oxpm`
    pub fn new() -> Result<Self> {
        let path = if let Some(env) = std::env::var_os("OXPM_HOME") {
            PathBuf::from(env)
        } else {
            let home = dirs_home().ok_or(Error::HomeNotFound)?;
            home.join(".oxpm")
        };
        Ok(Self { path })
    }

    /// 从指定路径创建
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// `~/.oxpm`
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `~/.oxpm/cache`
    pub fn cache(&self) -> PathBuf {
        self.path.join("cache")
    }

    /// 获取 store 管理器
    pub fn store(&self) -> OxpmStoreDir {
        OxpmStoreDir::new(&self.path)
    }

    /// `~/.oxpm/cache/git`
    pub fn git_cache(&self) -> PathBuf {
        self.cache().join("git")
    }

    /// `~/.oxpm/cache/tarball`
    pub fn tarball_cache(&self) -> PathBuf {
        self.cache().join("tarball")
    }

    /// 递归创建所有必要目录
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(self.cache())?;
        std::fs::create_dir_all(self.git_cache())?;
        std::fs::create_dir_all(self.tarball_cache())?;
        std::fs::create_dir_all(self.store().path())?;
        Ok(())
    }
}

/// `~/.oxpm/store/v1/` 内容寻址目录
#[derive(Debug, Clone)]
pub struct OxpmStoreDir {
    path: PathBuf,
}

impl OxpmStoreDir {
    pub fn new(home_path: &Path) -> Self {
        Self {
            path: home_path.join("store").join(STORE_VERSION),
        }
    }

    /// `~/.oxpm/store/v1/`
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `~/.oxpm/store/v1/<blake3_hash>/`
    pub fn package_path(&self, hash: &str) -> PathBuf {
        self.path.join(hash)
    }

    /// 包是否已在 store 中
    pub fn exists(&self, hash: &str) -> bool {
        self.package_path(hash).exists()
    }
}

fn dirs_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oxpm_dir_from_path() {
        let dir = OxpmDir::from_path("/tmp/test-oxpm");
        assert!(dir.path().ends_with("test-oxpm"));
        assert!(dir.cache().ends_with("cache"));
        assert!(dir.git_cache().ends_with("git"));
        assert!(dir.tarball_cache().ends_with("tarball"));
    }

    #[test]
    fn store_dir_path() {
        let dir = OxpmDir::from_path("/tmp/test-oxpm");
        let store = dir.store();
        assert!(store.path().ends_with("store/v1"));
    }

    #[test]
    fn store_package_path() {
        let dir = OxpmDir::from_path("/tmp/test-oxpm");
        let store = dir.store();
        let hash = "abc123def456";
        assert!(store.package_path(hash).ends_with(hash));
    }

    #[test]
    fn store_exists_returns_false_for_missing() {
        let dir = OxpmDir::from_path("/tmp/nonexistent-oxpm-test");
        let store = dir.store();
        assert!(!store.exists("fakehash"));
    }

    #[test]
    fn ensure_dirs_creates_structure() {
        let tmp = std::env::temp_dir().join("oxpm_path_test");
        let dir = OxpmDir::from_path(&tmp);
        dir.ensure_dirs().unwrap();
        assert!(tmp.join("cache/git").is_dir());
        assert!(tmp.join("cache/tarball").is_dir());
        assert!(tmp.join("store/v1").is_dir());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
