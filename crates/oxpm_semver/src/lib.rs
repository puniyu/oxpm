mod error;
pub use error::Error;
mod range;
pub use range::{VersionRange, VersionRangeKind};
mod version;
pub use version::Version;

pub type Result<T> = std::result::Result<T, Error>;