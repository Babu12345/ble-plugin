// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Defined errors for this library

use thiserror_no_std::Error;

/// Crate errors
#[derive(Debug, Error)]
pub enum HostError {
    /// Custom error. Usually just for testing
    #[error("Custom error with message {0}")]
    Custom(&'static str),
}

/// Result type for the configured crate errors
pub type HostResult<T> = core::result::Result<T, HostError>;
