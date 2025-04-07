//! Contains errors for the crate

/// Crate errors
pub enum PluginError {}

/// Result type for the configured crate errors
pub type PluginResult<T> = core::result::Result<T, PluginError>;
