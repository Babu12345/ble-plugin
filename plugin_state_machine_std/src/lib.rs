#![deny(missing_docs)]
// use protoco
//! This library is used to to contain the complete processing logic and state machine to facilitate data/command transfer from BLE
//! to usb and visa versa.

use esp32_nimble::BLEDevice;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
/// Contains state machine to process BLE and usb data and facilitate their data transfer
pub struct PluginStateMachine {
    usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
    usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
    ble_device: BLEDevice,
}

impl PluginStateMachine {
    /// Create a new instance of the processing state machine
    pub fn new(
        usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
        usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
        ble_device: BLEDevice,
    ) -> Self {
        Self {
            usb_sender,
            usb_receiver,
            ble_device,
        }
    }
}

#[cfg(test)]
mod tests {}
