// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Util functions
// TODO: Remove this once the methods are used in the crate
#![allow(unused)]

use std::sync::Arc;

use esp32_nimble::{BLEAddressType, BLEDevice};
use plugin_config::{plugin::PluginSender, BluetoothAddressType, PluginData, DEFAULT_PACKET_SIZE};

use crate::errors::{Error, Result};

pub fn bluetooth_address_type_to_ble_address_type(
    address_type: BluetoothAddressType,
) -> BLEAddressType {
    match address_type {
        BluetoothAddressType::Public => BLEAddressType::Public,
        BluetoothAddressType::Random => BLEAddressType::Random,
        BluetoothAddressType::PublicId => BLEAddressType::PublicID,
        BluetoothAddressType::RandomId => BLEAddressType::RandomID,
        _ => unreachable!(),
    }
}

pub fn ble_address_type_to_bluetooth_address_type(
    address_type: BLEAddressType,
) -> BluetoothAddressType {
    match address_type {
        BLEAddressType::Public => BluetoothAddressType::Public,
        BLEAddressType::Random => BluetoothAddressType::Random,
        BLEAddressType::PublicID => BluetoothAddressType::PublicId,
        BLEAddressType::RandomID => BluetoothAddressType::RandomId,
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
            Error::UsbSendError
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
            Error::UsbSendError
        })?;
    }
    Ok(())
}

pub fn set_device_name(name: &str) {
    BLEDevice::set_device_name(name).ok();
}
