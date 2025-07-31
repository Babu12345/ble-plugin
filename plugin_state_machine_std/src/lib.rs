#![deny(missing_docs)]
//! This library is used to to contain the complete processing logic and state machine to facilitate data/command transfer from BLE
//! to usb and visa versa.

use std::time::Duration;

use esp32_nimble::BLEDevice;
use protocol::DEFAULT_PACKET_SIZE;
use protocol::io_types::{HostCommandConfigurePeripheral, HostCommandConfigureService};
use protocol::plugin::plugin::{PluginReceiver, PluginSender};
/// Contains state machine to process BLE and usb data and facilitate their data transfer
pub struct PluginStateMachine {
    usb_sender: PluginSender<DEFAULT_PACKET_SIZE>,
    usb_receiver: PluginReceiver<DEFAULT_PACKET_SIZE>,
    ble_device: BLEDevice,
}

/// There will be 2 runners the first will be processing
/// usb data and sending it to BLE. The second will be processing
/// BLE data and sending it to USB.
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

    /// This runner will process the USB data and send it to BLE.
    /// This should usually be run in a thread to not block the main thread.
    /// TODO: Make sure that you can only serialize one possibly command per data as to not cause
    /// incorrect deserialization
    pub fn usb_to_ble_runner(&mut self) {
        loop {
            match self.usb_receiver.receive() {
                Ok(data) => {
                    let maybe_cmd: Option<HostCommandConfigurePeripheral> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                    }

                    let maybe_cmd: Option<HostCommandConfigureService> = data.decode().ok();
                    if let Some(cmd) = maybe_cmd {
                        log::info!("Received USB command: {:?}", cmd);
                    }
                }
                Err(_) => {
                    // Handle error, possibly log or retry
                    log::error!("Failed to receive data from USB");
                    // Add in a sleep so to not take up too much CPU time if there are repeated errors
                    std::thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }
    }

    /// This runner will process the BLE data and send it to USB.
    /// Processing should only happen once the BLE device has been setup and connected
    /// This should usually be run in a thread to as to not block the main thread
    pub fn ble_to_usb_runner(&mut self) {
        loop {}
    }
}

#[cfg(test)]
mod tests {}
