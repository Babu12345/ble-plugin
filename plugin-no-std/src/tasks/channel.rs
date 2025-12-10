// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Defines the communication channels

pub use crate::configs::BUFFER_SIZE;

/// Channel send and receive type
pub type TChannel = [u8; BUFFER_SIZE];
/// Channel size
pub const CHANNEL_SIZE: usize = 100;
