mod cache;
mod error;
pub use error::Error;
mod version;
mod source;
mod dep;
mod types;
pub use types::*;
mod resolver;
pub use resolver::Resolver;

pub type Result<T> = std::result::Result<T, Error>;
