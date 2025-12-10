// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Error library

use esp32_nimble::BLEError;

use thiserror_no_std::Error as ThisError;

/// Comprehensive error types for the esp-nimble plugin config
///
/// This enum covers all possible error conditions that can occur during
/// BLE-USB bridge operations, from low-level communication failures to
/// high-level protocol violations.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Invalid BLE peripheral, service, or characteristic configuration
    ///
    /// This error occurs when:
    /// - Attempting to create services without configuring peripheral first
    /// - Trying to create characteristics without service configuration
    /// - Using invalid UUIDs or property combinations
    /// - Accessing non-existent services or characteristics
    #[error(
        "Invalid BLE configuration - check peripheral, service, and characteristic setup order"
    )]
    InvalidBleConfiguration,

    /// BLE advertisement setup or operation failure
    ///
    /// Contains a static string describing the specific advertisement error.
    /// Common causes include:
    /// - Advertisement data too large
    /// - BLE stack not ready
    /// - Hardware communication failure
    #[error("BLE advertisement error: {0}")]
    AdvertisementError(&'static str),

    /// BLE server not initialized when operation requires it
    ///
    /// This error indicates that a BLE operation was attempted before
    /// the peripheral was properly configured. The peripheral must be
    /// configured first to initialize the BLE server.
    #[error("BLE server not initialized - configure peripheral first")]
    ServerNotInitialized,

    /// USB communication failure when sending data to host
    ///
    /// This error occurs when the USB channel is unable to send
    /// responses or data back to the host. Common causes include:
    /// - USB connection lost
    /// - Channel buffer full
    /// - Host not ready to receive
    #[error("Failed to send data over USB channel")]
    UsbSendError,

    /// BLE characteristic notification or indication failure
    ///
    /// This error occurs when attempting to notify or indicate a
    /// characteristic value to connected clients. Common causes include:
    /// - Client not subscribed to notifications
    /// - Connection lost
    /// - Characteristic not configured for notifications
    #[error("Failed to notify characteristic value to connected clients")]
    CharacteristicNotificationError,

    /// Failure to store characteristic UUID in internal metadata
    ///
    /// This error indicates that the internal storage structures are full
    /// or corrupted. It typically occurs when trying to create more
    /// characteristics than the embedded system can handle.
    #[error("Failed to store characteristic UUID - storage full or corrupted")]
    CharacteristicUuidStorageError,

    /// USB message format validation failure
    ///
    /// This error occurs when received USB data doesn't conform to the
    /// expected message protocol. Common causes include:
    /// - Invalid magic number
    /// - Insufficient header size
    /// - Corrupted data transmission
    #[error("Invalid USB message format - check magic number and header structure")]
    InvalidMessageFormat,

    /// Failure to set the random address for the BLE device
    #[error("Failed to set random address")]
    UnableToSetRNDAddress,

    /// Invalid passkey length provided for BLE pairing
    #[error("Invalid passkey length - must be 6 digits")]
    InvalidPasskeyLength,

    /// Failure to restart the BLE server
    #[error("Failed to restart BLE server with error {0}")]
    ServerRestartError(#[source] BLEError),

    // NVS related errors
    /// Failure to save data to NVS storage
    #[error("Failed to resolve NVS namespace")]
    FailedToResolveNvsNamespace,

    /// Failure to write data to NVS storage
    #[error("Failed to write data to NVS storage")]
    NvsWriteError,

    /// Failure to read data from NVS storage
    #[error("Failed to read data from NVS storage")]
    NvsReadError,
}

/// Result type for the library
pub type Result<T> = core::result::Result<T, Error>;
