mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod format;
pub use format::CompressionFormat;

mod extract;
pub use extract::Extract;

mod compress;
pub use compress::Compress;
