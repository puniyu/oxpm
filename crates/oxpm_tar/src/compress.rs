use std::io::Write;
use std::path::Path;

use super::error::Error;
use super::format::CompressionFormat;

#[derive(Debug, Default)]
pub struct Compress {
    pub format: CompressionFormat,
    pub follow_symlinks: bool,
}

impl Compress {
    pub fn with_format(mut self, format: CompressionFormat) -> Self {
        self.format = format;
        self
    }

    pub fn follow_symlinks(mut self, yes: bool) -> Self {
        self.follow_symlinks = yes;
        self
    }

    pub fn compress(&self, src: &Path, dest: &Path) -> super::Result<()> {
        let file = std::fs::File::create(dest).map_err(Error::Io)?;
        let writer = std::io::BufWriter::new(file);
        self.compress_writer(src, writer)
    }

    pub fn compress_writer<W: Write + 'static>(&self, src: &Path, writer: W) -> super::Result<()> {
        let encoder = make_encoder(writer, self.format)?;
        let mut builder = tar::Builder::new(encoder);
        builder.follow_symlinks(self.follow_symlinks);

        for entry in walkdir::WalkDir::new(src) {
            let entry = entry.map_err(|e| std::io::Error::other(e.to_string()))?;
            let path = entry.path();
            let relative = path.strip_prefix(src).map_err(|e| std::io::Error::other(e.to_string()))?;

            if path.is_file() {
                let mut file = std::fs::File::open(path).map_err(Error::Io)?;
                builder.append_file(relative, &mut file).map_err(Error::Io)?;
            } else if path.is_dir() && !relative.as_os_str().is_empty() {
                builder.append_dir(relative, path).map_err(Error::Io)?;
            }
        }

        builder.finish().map_err(Error::Io)?;
        Ok(())
    }
}

fn make_encoder<W: Write + 'static>(writer: W, format: CompressionFormat) -> super::Result<Box<dyn Write>> {
    match format {
        CompressionFormat::Auto | CompressionFormat::Gzip => {
            let encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
            Ok(Box::new(encoder) as Box<dyn Write>)
        }
        #[cfg(feature = "zstd")]
        CompressionFormat::Zstd => {
            let encoder =
                zstd::stream::write::Encoder::new(writer, 0).map_err(|e| Error::Archive(e.to_string()))?
                    .auto_finish();
            Ok(Box::new(encoder) as Box<dyn Write>)
        }
        #[cfg(feature = "bz2")]
        CompressionFormat::Bz2 => {
            let encoder = bzip2::write::BzEncoder::new(writer, bzip2::Compression::default());
            Ok(Box::new(encoder) as Box<dyn Write>)
        }
    }
}