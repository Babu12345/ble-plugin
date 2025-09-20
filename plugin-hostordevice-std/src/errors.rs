//! Crate error and result types

use crate::resolver::USBTypeResolver;

#[derive(Debug)]
/// Error type for the plugin device
pub enum PluginError {
    /// Error indicating that the peripherals could not be taken
    PeripheralsUnavailable,
    // GPIO errors
    /// Error indicating that a GPIO pin could not be initialized
    GpioInitError(&'static str),
    /// Error indicating a GPIO operation failed
    GpioOperationError(&'static str),
    /// Error indicating that the USB host or device could not be initialized
    UsbInitError(USBTypeResolver),
    /// Proccessor initialization error
    ProcessorInitError(USBTypeResolver),
}

/// Result type for the plugin, using the PluginError type
pub type Result<T> = std::result::Result<T, PluginError>;
