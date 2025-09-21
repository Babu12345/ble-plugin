//! Util functions for the state machine

use esp32_nimble::BLEAddressType;

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
