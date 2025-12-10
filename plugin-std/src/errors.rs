// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

use thiserror_no_std::Error;

#[derive(Debug, Error)]
/// Error type for the plugin
pub enum PluginError {
    /// Error indicating that the peripherals could not be taken
    #[error("Failed to take peripherals")]
    PeripheralsUnavailable,
    /// Error indicating that a GPIO pin could not be initialized
    #[error("Failed to initialize GPIO {0}")]
    GpioInitError(&'static str),
    /// Error indicating a GPIO operation failed
    #[error("GPIO operation failed: {0}")]
    GpioOperationError(&'static str),
    /// Error indicating that the USB device could not be initialized
    #[error("USB device initialization failed: {0}")]
    UsbDeviceInitError(&'static str),
    /// Error indicating that a USB processor encountered an error
    #[error("USB processor error: {0}")]
    UsbProcessorError(&'static str),
    /// Error indicating that the BLE device encountered an error
    #[error("BLE device error: {0}")]
    BleDeviceError(&'static str),
}

/// Result type for the plugin, using the PluginError type
pub type Result<T> = std::result::Result<T, PluginError>;
