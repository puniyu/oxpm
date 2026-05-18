use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CompressionFormat {
    #[default]
    Auto,
    Gzip,
    #[cfg(feature = "zstd")]
    Zstd,
    #[cfg(feature = "bz2")]
    Bz2,
}

impl CompressionFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "tgz" | "tar.gz" | "gz" => CompressionFormat::Gzip,
            #[cfg(feature = "zstd")]
            "tar.zst" | "tzst" => CompressionFormat::Zstd,
            #[cfg(feature = "bz2")]
            "tar.bz2" | "tbz2" | "tbz" | "bz2" => CompressionFormat::Bz2,
            _ => CompressionFormat::Gzip,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(CompressionFormat::Gzip)
    }
}