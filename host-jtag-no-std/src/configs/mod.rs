// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Configuration functions for this library

use crate::error::HostResult;

/// Initialize the ESP32 logger capbilities
pub fn initalize_logger() -> HostResult<()> {
    esp_println::logger::init_logger(log::LevelFilter::Trace);
    Ok(())
}
