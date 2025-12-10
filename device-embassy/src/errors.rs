// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Error and result types for the crate

/// Error types
#[derive(Debug)]
pub enum Error {}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
