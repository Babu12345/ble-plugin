// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Error types for this library

use thiserror_no_std::Error;

/// Library error type
#[derive(Debug, Error)]
pub enum Error {
    /// Custom error type
    #[error("Custom error: {0}")]
    CustomError(&'static str),
    /// Reinitialization is attempted
    #[error(
        "The usb device is already initialized. Please deinitialize or call one of the processors"
    )]
    DeviceAlreadyInitialized,
    /// Device initialization failed
    #[error("Device failed to initialize")]
    InitializationFailure,
    /// The busid isn't defined
    #[error("The busid wasn't defined during initialization. Please do so.")]
    BusidUndefined,
}

/// Result type with the custom error
pub type Result<T> = core::result::Result<T, Error>;
