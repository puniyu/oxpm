mod error;
pub use error::*;

mod source;
pub use source::*;

pub type Result<T> = std::result::Result<T, Error>;