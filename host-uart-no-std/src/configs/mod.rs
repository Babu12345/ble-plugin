//! Configuration functions for this library

use crate::error::HostResult;

/// Initialize the ESP32 logger capbilities
pub fn initalize_logger() -> HostResult<()> {
    esp_println::logger::init_logger(log::LevelFilter::Trace);
    Ok(())
}
