//! Initialize hardware configurations

use anyhow::Result;
/// Initialize the ESP32 logger capbilities
pub fn initalize_logger() -> Result<()> {
    esp_println::logger::init_logger(log::LevelFilter::Trace);
    Ok(())
}
