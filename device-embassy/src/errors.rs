//! Error and result types for the crate

/// Error types
pub struct Error {}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
