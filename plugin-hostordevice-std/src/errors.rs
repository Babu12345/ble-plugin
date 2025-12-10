// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

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
