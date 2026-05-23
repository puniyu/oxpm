use oxpm_source::Source;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use sugar_path::SugarPath;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TarballType {
    Http(Url),
    Https(Url),
    File(SmolStr),
}

impl TarballType {
    pub fn from_url(url: &Url) -> Self {
        match url.scheme() {
            "http" => TarballType::Http(url.clone()),
            "https" => TarballType::Https(url.clone()),
            "file" => TarballType::File(url.path().into()),
            _ => TarballType::File(url.as_str().into()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TarballType::Http(url) => url.as_str(),
            TarballType::Https(url) => url.as_str(),
            TarballType::File(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TarballSource {
    tarball_type: TarballType,
}

impl TarballSource {
    pub fn new(tarball_type: TarballType) -> Self {
        Self { tarball_type }
    }

    pub fn tarball_type(&self) -> &TarballType {
        &self.tarball_type
    }

    pub fn path(&self) -> &str {
        self.tarball_type.as_str()
    }

    pub fn as_str(&self) -> &str {
        self.tarball_type.as_str()
    }

    pub fn is_absolute(&self) -> bool {
        match &self.tarball_type {
            TarballType::File(s) => std::path::Path::new(s.as_str()).is_absolute(),
            _ => true,
        }
    }

    pub fn is_relative(&self) -> bool {
        match &self.tarball_type {
            TarballType::File(s) => std::path::Path::new(s.as_str()).is_relative(),
            _ => false,
        }
    }

    pub fn to_absolute(&self, base: &str) -> SmolStr {
        match &self.tarball_type {
            TarballType::File(path) => {
                path.as_str().absolutize_with(base).to_string_lossy().into()
            }
            _ => SmolStr::new(self.tarball_type.as_str()),
        }
    }

    pub fn from_path(path: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        Ok(Self::new(TarballType::File(path.to_string_lossy().into_owned().into())))
    }

    pub fn from_url(url: &Url) -> Self {
        Self::new(TarballType::from_url(url))
    }
}

impl Source for TarballSource {
    fn name(&self) -> &'static str {
        "Tarball"
    }

    fn as_str(&self) -> SmolStr {
        SmolStr::new(format!("tarball:{}", self.tarball_type.as_str()))
    }
}

impl std::fmt::Display for TarballSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tarball_type.as_str())
    }
}

impl AsRef<str> for TarballSource {
    fn as_ref(&self) -> &str {
        self.tarball_type.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let source = TarballSource::new(TarballType::File("./dist/my-pkg-1.0.0.tgz".into()));
        assert_eq!(format!("{}", source), "./dist/my-pkg-1.0.0.tgz");
    }

    #[test]
    fn tarball_type_http() {
        let url = Url::parse("http://example.com/pkg.tgz").unwrap();
        let tt = TarballType::from_url(&url);
        assert!(matches!(tt, TarballType::Http(_)));
        assert_eq!(tt.as_str(), "http://example.com/pkg.tgz");
    }

    #[test]
    fn tarball_type_https() {
        let url = Url::parse("https://registry.npmjs.org/pkg-1.0.0.tgz").unwrap();
        let tt = TarballType::from_url(&url);
        assert!(matches!(tt, TarballType::Https(_)));
        assert_eq!(tt.as_str(), "https://registry.npmjs.org/pkg-1.0.0.tgz");
    }

    #[test]
    fn tarball_type_file() {
        let url = Url::parse("file:./packages/my-pkg.tgz").unwrap();
        let tt = TarballType::from_url(&url);
        assert!(matches!(tt, TarballType::File(_)));
        assert!(tt.as_str().ends_with("packages/my-pkg.tgz"));
    }
}