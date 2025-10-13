//! Util functions for the state machine

use std::sync::Arc;

use esp32_nimble::{BLEAddressType, BLEDevice};
use protocol::{plugin::plugin::PluginSender, protocol::PluginData};

use crate::errors::{Result, StateMachineError};
use protocol::DEFAULT_PACKET_SIZE;

pub fn bluetooth_address_type_to_ble_address_type(
    address_type: protocol::protocol::BluetoothAddressType,
) -> BLEAddressType {
    match address_type {
        protocol::protocol::BluetoothAddressType::Public => BLEAddressType::Public,
        protocol::protocol::BluetoothAddressType::Random => BLEAddressType::Random,
        protocol::protocol::BluetoothAddressType::PublicId => BLEAddressType::PublicID,
        protocol::protocol::BluetoothAddressType::RandomId => BLEAddressType::RandomID,
        _ => unreachable!(),
    }
}

pub fn ble_address_type_to_bluetooth_address_type(
    address_type: BLEAddressType,
) -> protocol::protocol::BluetoothAddressType {
    match address_type {
        BLEAddressType::Public => protocol::protocol::BluetoothAddressType::Public,
        BLEAddressType::Random => protocol::protocol::BluetoothAddressType::Random,
        BLEAddressType::PublicID => protocol::protocol::BluetoothAddressType::PublicId,
        BLEAddressType::RandomID => protocol::protocol::BluetoothAddressType::RandomId,
    }
}

/// Send plugin data with automatic chunking if data exceeds max_plugin_data_send limit
pub fn send_plugin_data_chunked(
    sender: Arc<PluginSender<DEFAULT_PACKET_SIZE>>,
    plugin_data: PluginData,
    max_plugin_data_send_size: usize,
) -> Result<()> {
    let plugin_data_length = plugin_data.data.len();
    if plugin_data_length <= max_plugin_data_send_size {
        // Data fits in a single message, send as-is
        sender.send(plugin_data).map_err(|_| {
            log::error!("Failed to send plugin data");
            StateMachineError::UsbSendError
        })?;
        log::trace!("Sent plugin data: {} bytes", plugin_data_length);
        return Ok(());
    }
    // Data needs to be chunked
    let total_chunks = plugin_data_length.div_ceil(max_plugin_data_send_size);

    for (chunk_index, chunk) in plugin_data
        .data
        .chunks(max_plugin_data_send_size)
        .enumerate()
    {
        let mut chunk_data = plugin_data.clone();
        chunk_data.data = chunk.to_vec();

        sender.send(chunk_data).map_err(|_| {
            log::error!(
                "Failed to send plugin data chunk {}/{}",
                chunk_index + 1,
                total_chunks
            );
            StateMachineError::UsbSendError
        })?;
    }
    Ok(())
}

pub fn set_device_name(name: &str) {
    BLEDevice::set_device_name(name).ok();
}
