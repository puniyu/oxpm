use std::io::Read;
use std::path::Path;

use super::error::Error;
use super::format::CompressionFormat;

#[derive(Debug, Default)]
pub struct Extract {
    pub format: CompressionFormat,
}

impl Extract {
    pub fn with_format(mut self, format: CompressionFormat) -> Self {
        self.format = format;
        self
    }

    pub fn extract(&self, src: &Path, dest: &Path) -> super::Result<()> {
        let reader = std::fs::File::open(src).map_err(Error::Io)?;
        let reader = std::io::BufReader::new(reader);
        self.extract_reader(reader, dest)
    }

    pub fn extract_reader<R: Read + 'static>(&self, reader: R, dest: &Path) -> super::Result<()> {
        let decoder = make_decoder(reader, self.format)?;
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(dest).map_err(|e| Error::Archive(e.to_string()))?;
        Ok(())
    }

    pub fn read_file(&self, src: &Path, file_path: &str) -> super::Result<String> {
        let reader = std::fs::File::open(src).map_err(Error::Io)?;
        let reader = std::io::BufReader::new(reader);
        let decoder = make_decoder(reader, self.format)?;
        let mut archive = tar::Archive::new(decoder);
        for entry in archive.entries().map_err(|e| Error::Archive(e.to_string()))? {
            let mut entry = entry.map_err(|e| Error::Archive(e.to_string()))?;
            let entry_path = entry.path().map_err(|e| Error::Archive(e.to_string()))?;
            let entry_str = entry_path.to_str().ok_or_else(|| Error::Archive("invalid path".into()))?;
            if entry_str == file_path || entry_str == format!("package/{}", file_path) {
                let mut contents = String::new();
                entry.read_to_string(&mut contents).map_err(Error::Io)?;
                return Ok(contents);
            }
        }
        Err(Error::Archive(format!("file not found in tarball: {}", file_path)))
    }
}

fn make_decoder<R: Read + 'static>(reader: R, format: CompressionFormat) -> super::Result<Box<dyn Read>> {
    match format {
        CompressionFormat::Auto | CompressionFormat::Gzip => {
            let decoder = flate2::read::GzDecoder::new(reader);
            Ok(Box::new(decoder) as Box<dyn Read>)
        }
        #[cfg(feature = "zstd")]
        CompressionFormat::Zstd => {
            let decoder =
                zstd::stream::read::Decoder::new(reader).map_err(|e| Error::Archive(e.to_string()))?;
            Ok(Box::new(decoder) as Box<dyn Read>)
        }
        #[cfg(feature = "bz2")]
        CompressionFormat::Bz2 => {
            let decoder = bzip2::read::BzDecoder::new(reader);
            Ok(Box::new(decoder) as Box<dyn Read>)
        }
    }
}