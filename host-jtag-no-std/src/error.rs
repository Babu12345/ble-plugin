//! Defined errors for this library

use thiserror_no_std::Error;

/// Crate errors
#[derive(Debug, Error)]
pub enum HostError {
    /// Custom error. Usually just for testing
    #[error("Custom error with message {0}")]
    Custom(&'static str),
}

/// Result type for the configured crate errors
pub type HostResult<T> = core::result::Result<T, HostError>;
