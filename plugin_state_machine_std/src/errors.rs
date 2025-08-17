//! Error types and result definitions for the plugin state machine
//!
//! This module defines comprehensive error handling for the BLE-USB bridge state machine,
//! covering USB communication errors, BLE configuration issues, and message processing failures.
//!
//! ## Error Categories
//!
//! - **Communication Errors**: USB send failures and data processing issues
//! - **BLE Configuration Errors**: Invalid peripheral, service, or characteristic setup
//! - **Message Protocol Errors**: Malformed messages and unknown command types
//! - **State Management Errors**: Server initialization and storage failures
//!
//! ## Usage
//!
//! ```rust,no_run
//! use plugin_state_machine_std::errors::{Result, StateMachineError};
//!
//! fn process_command() -> Result<()> {
//!     // Operation that might fail
//!     Err(StateMachineError::InvalidBleConfiguration)
//! }
//!
//! match process_command() {
//!     Ok(()) => println!("Success"),
//!     Err(StateMachineError::InvalidBleConfiguration) => {
//!         println!("BLE configuration error");
//!     }
//!     Err(e) => println!("Other error: {:?}", e),
//! }
//! ```

use esp32_nimble::BLEError;
use protocol::MessageTypeId;
use thiserror_no_std::Error;

/// Comprehensive error types for plugin state machine operations
///
/// This enum covers all possible error conditions that can occur during
/// BLE-USB bridge operations, from low-level communication failures to
/// high-level protocol violations.
#[derive(Debug, Error)]
pub enum StateMachineError {
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

    /// General data processing error with descriptive message
    ///
    /// Contains a static string describing the specific processing error.
    /// Used for various data handling failures that don't fit other categories.
    #[error("Data processing error: {0}")]
    DataProcessingError(&'static str),

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

    /// Failure to decode USB message into expected command type
    #[error("Failed to decode USB message into command type: {0}")]
    FailedToDecodeMessage(&'static str),

    /// Unhandled message type received from host
    #[error("Unhandled message type received from host")]
    UnhandledMessageType(MessageTypeId),

    /// Unknown or unsupported message type ID
    ///
    /// This error indicates that the message type ID extracted from the
    /// USB data doesn't correspond to any known command type. This may
    /// indicate protocol version mismatch or data corruption.
    #[error("Unknown message type ID - possible protocol version mismatch")]
    UnknownMessageType,

    /// Failure to set the random address for the BLE device
    #[error("Failed to set random address")]
    UnableToSetRNDAddress,

    /// Invalid passkey length provided for BLE pairing
    #[error("Invalid passkey length - must be 6 digits")]
    InvalidPasskeyLength,

    /// Failure to restart the BLE server
    #[error("Failed to restart BLE server with error {0}")]
    ServerRestartError(#[source] BLEError),
}

/// Convenient result type for plugin state machine operations
///
/// This type alias provides a standard `Result<T, StateMachineError>` for
/// all operations in the plugin state machine, simplifying error handling
/// and function signatures throughout the crate.
///
/// # Examples
///
/// ```rust,no_run
/// use plugin_state_machine_std::errors::{Result, StateMachineError};
///
/// fn configure_peripheral(name: &str) -> Result<()> {
///     if name.is_empty() {
///         return Err(StateMachineError::InvalidBleConfiguration);
///     }
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, StateMachineError>;
