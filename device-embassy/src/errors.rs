//! Error and result types for the crate

/// Error types
#[derive(Debug)]
pub enum Error {}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
