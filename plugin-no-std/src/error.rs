//! Contains errors for the crate

use thiserror_no_std::Error;

/// Crate errors
#[derive(Debug, Error)]
pub enum PluginError {
    /// Custom error. Usually just for testing
    #[error("Custom error with message {0}")]
    Custom(&'static str),
}

/// Result type for the configured crate errors
pub type PluginResult<T> = core::result::Result<T, PluginError>;
