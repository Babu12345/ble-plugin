// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Error types and result definitions for the plugin state machine
//!
//! This module defines comprehensive error handling for the BLE-USB bridge state machine,
//! covering USB communication errors, BLE configuration issues, and message processing failures.
//!
//! ## Error Categories
//!
//! - Failure to decode
//! - Unhandled message types
//! - Internal plugin configuration error

use protocol::protocol::MessageTypeId;
use thiserror_no_std::Error;

/// Comprehensive error types for plugin state machine operations
///
/// This enum covers all possible error conditions that can occur during
/// BLE-USB bridge operations, from low-level communication failures to
/// high-level protocol violations.
#[derive(Debug, Error)]
pub enum StateMachineError<ConfigError> {
    /// Failure to decode USB message into expected command type
    #[error("Failed to decode USB message into command type: {0}")]
    FailedToDecodeMessage(&'static str),
    /// Unhandled message type received from host
    #[error("Unhandled message type received from host")]
    UnhandledMessageType(MessageTypeId),
    /// Internal config error
    #[error("Internal error")]
    InternalConfigError(#[from] ConfigError),
}

/// Convenient result type for plugin state machine operations
///
/// This type alias provides a standard `Result<T, StateMachineError>` for
/// all operations in the plugin state machine, simplifying error handling
/// and function signatures throughout the crate.

pub type Result<T, E> = core::result::Result<T, StateMachineError<E>>;
