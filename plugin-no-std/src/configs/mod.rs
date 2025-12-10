// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Initialize hardware configurations

mod ble;
mod usb;
pub use ble::*;
pub use usb::*;

use crate::PluginResult;

/// Initialize the ESP32 logger capbilities
pub fn initalize_logger() -> PluginResult<()> {
    esp_println::logger::init_logger(log::LevelFilter::Trace);
    Ok(())
}
