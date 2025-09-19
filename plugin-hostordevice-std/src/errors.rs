//! Crate error and result types

#[derive(Debug)]
/// Error type for the plugin device
pub enum PluginError {
    /// Error indicating that the peripherals could not be taken
    PeripheralsUnavailable,
    /// Error indicating that a GPIO pin could not be initialized
    GpioInitError(&'static str),
    /// Error indicating a GPIO operation failed
    GpioOperationError(&'static str),
    /// Error indicating that the USB device could not be initialized
    UsbDeviceInitError(&'static str),
}

/// Result type for the plugin, using the PluginError type
pub type Result<T> = std::result::Result<T, PluginError>;
