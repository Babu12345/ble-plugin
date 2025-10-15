#![deny(missing_docs)]
//! Implements the plugin_config to be used in the plugin state machines for esp_nimble. Which is a bluetooth crate for esp32

pub mod errors;
use esp32_nimble::BLEDevice;
use heapless::String;
use plugin_config::PluginConfig;

use crate::errors::Error;

#[derive(Default)]
struct Metadata {
    ble_name: String<30>,
}
/// Nimble struct
pub struct EspNimble<'a> {
    is_initialized: bool,
    device: &'a mut BLEDevice,
    metadata: Metadata,
}

impl<'a> EspNimble<'a> {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            is_initialized: false,
            device: BLEDevice::take(),
            metadata: Default::default(),
        }
    }
    /// Initialize the struct
    pub fn initialize(&mut self) {
        self.is_initialized = true
    }
}

impl<'a> PluginConfig<Error> for EspNimble<'a> {}
