//! Error and result types for the crate

/// Error types
pub enum Error {}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
