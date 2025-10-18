//! Error library

/// Errors for the nimble plugin config
pub enum Error {
    // Nimble
    /// Unable to set the address
    UnableToSetRNDAddress,
    /// Server failed to be initialized
    ServerNotInitialized,
    /// Invalid configuration
    InvalidBleConfiguration,
    // USB
    /// Usb send error
    UsbSendError,
    // NVS
    /// Failed to resolve namespace
    FailedToResolveNvsNamespace,
    /// Write error
    NvsWriteError,
}

/// Result type for the library
pub type Result<T> = core::result::Result<T, Error>;
