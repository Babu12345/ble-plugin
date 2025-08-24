//! Error crate for the plugin-nvc crate.

/// Defines error types and handling mechanisms specific to non-volatile storage operations.
pub enum PluginNvcError {
    /// Namespace aquisition error.
    NamespaceAcquisitionError,
    /// Error indicating that the specified NVS namespace could not be found.
    NamespaceNotFound,
    /// Error indicating a failure to read from NVS.
    NvsReadError,
    /// Error indicating a failure to write to NVS.
    NvsWriteError,
    /// Error indicating a failure to erase data from NVS.
    NvsEraseError,
    /// Error indicating that the NVS partition is full.
    NvsPartitionFull,
    /// Generic error for other NVS-related issues.
    NvsGenericError,
}

/// Result type alias for operations that can return a PluginNvcError.
pub type Result<T> = core::result::Result<T, PluginNvcError>;
