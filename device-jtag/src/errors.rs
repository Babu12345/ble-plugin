//! Error types for this library

use thiserror_no_std::Error;

/// Library error type
#[derive(Debug, Error)]
pub enum Error {
    /// Custom error type
    #[error("Custom error: {0}")]
    CustomError(&'static str),
}

/// Result type with the custom error
pub type Result<T> = core::result::Result<T, Error>;
