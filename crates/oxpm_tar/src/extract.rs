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