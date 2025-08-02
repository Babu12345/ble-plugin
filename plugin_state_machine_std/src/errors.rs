//! This file contains the error types and result type used in the state machine.

#[derive(Debug)]
/// Represents errors that can occur in the plugin state machine.
pub enum StateMachineError {
    /// Error related to invalid BLE configuration.
    InvalidBleConfiguration,
    /// Error related to BLE advertisement.
    AdvertisementError(&'static str),
    /// Error related to server initialization.
    ServerNotInitialized,
    /// Error related to USB sending.
    UsbSendError,
    /// Error related to USB processing.
    DataProcessingError(&'static str),
}

/// Result type for the plugin state machine operations.
pub type Result<T> = std::result::Result<T, StateMachineError>;
