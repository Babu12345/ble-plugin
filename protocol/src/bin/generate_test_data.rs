// Test data generation binary for protocol regression tests
// This file creates independent test structures and generates serialized binary data
// for use in Python regression tests in the plugin_host library. Run this file
// to regenerate test data after any changes to the serialization logic. This uses test
// structures that are independent of the main codebase to ensure no accidental
// dependencies are introduced.

use heapless::{String as HeaplessString, Vec as HeaplessVec};
use protocol::protocol::MessageTypeId;
use protocol::{HostIO, MessageType, PluginIO, DEFAULT_PACKET_SIZE, IO};
use protocol_io::{HostIO as HostIOMacro, PluginIO as PluginIOMacro};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use uuid::Uuid;

// ====== TEST CONSTANTS ======
const TEST_MAX_NAME_SIZE: usize = 20;
const TEST_MAX_PROPERTIES: usize = 3;
const TEST_MAX_CHARACTERISTICS: usize = 8;

// ====== TEST ENUMS ======

/// Test enum for BLE properties - independent of main codebase
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BLEPropertiesTest {
    TestRead = 10,
    TestWrite = 11,
    TestNotify = 12,
    TestIndicate = 13,
}

/// Test enum for bluetooth address types - independent of main codebase
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BluetoothAddressTypeTest {
    TestPublic = 20,
    TestRandom = 21,
    TestResolvable = 22,
}

/// Test enum for data send types - independent of main codebase
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DataSendTypeTest {
    TestNotification = 30,
    TestRead = 31,
    TestWrite = 32,
}

/// Test enum for configuration errors - independent of main codebase
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ConfigurationErrorTest {
    TestNameTooLong = 40,
    TestInvalidUuid = 41,
    TestServiceMissing = 42,
    TestCharacteristicMissing = 43,
}

// ====== TEST HOST COMMAND STRUCTURES ======

/// Test host command for peripheral configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandConfigurePeripheral)]
pub struct HostCommandConfigurePeripheralTest {
    pub test_name: HeaplessString<TEST_MAX_NAME_SIZE>,
    pub test_uuid: Uuid,
    pub test_enabled: bool,
    pub test_power_level: u8,
}

/// Test host command for service configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandConfigureService)]
pub struct HostCommandConfigureServiceTest {
    pub test_service_uuid: Uuid,
    pub test_priority: u8,
    pub test_visible: bool,
}

/// Test host command for characteristic configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandConfigureCharacteristic)]
pub struct HostCommandConfigureCharacteristicTest {
    pub test_char_uuid: Uuid,
    pub test_service_uuid: Uuid,
    pub test_properties: HeaplessVec<BLEPropertiesTest, TEST_MAX_PROPERTIES>,
    pub test_security_level: u8,
}

/// Test host command for characteristic read configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandConfigureCharacteristicRead)]
pub struct HostCommandConfigureCharacteristicReadTest {
    pub test_char_uuid: Uuid,
    pub test_service_uuid: Uuid,
    pub test_default_value: HeaplessVec<u8, 16>,
    pub test_read_permissions: u8,
}

/// Test host command for service info query
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandGetServiceInfo)]
pub struct HostCommandGetServiceInfoTest {
    pub test_service_uuid: Uuid,
    pub test_include_characteristics: bool,
}

/// Test host command for characteristic info query
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandGetCharacteristicInfo)]
pub struct HostCommandGetCharacteristicInfoTest {
    pub test_char_uuid: Uuid,
    pub test_service_uuid: Uuid,
    pub test_detailed_info: bool,
}

/// Test host command for advertisement start
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandStartAdvertisement)]
pub struct HostCommandStartAdvertisementTest {
    pub test_allow_multi_connect: bool,
    pub test_advertisement_interval: u8,
    pub test_tx_power: u8,
}

/// Test host command for characteristic value notification
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandNotifyCharacteristicValue)]
pub struct HostCommandNotifyCharacteristicValueTest {
    pub test_device_address: HeaplessVec<u8, 6>,
    pub test_address_type: BluetoothAddressTypeTest,
    pub test_char_uuid: Uuid,
    pub test_service_uuid: Uuid,
    pub test_notification_value: HeaplessVec<u8, 20>,
    pub test_confirm_required: bool,
}

// ====== TEST PLUGIN RESPONSE STRUCTURES ======

/// Test plugin response for data forwarding
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginData)]
pub struct PluginDataTest<'a> {
    pub test_source_id: Uuid,
    pub test_send_type: DataSendTypeTest,
    pub test_payload: &'a [u8],
    pub test_timestamp: u8,
    pub test_connection_handle: u8,
}

/// Test plugin configuration error response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginConfigurationError)]
pub struct PluginConfigurationErrorTest {
    pub test_error_type: ConfigurationErrorTest,
    pub test_error_code: u8,
    pub test_error_description: HeaplessString<32>,
}

/// Test plugin service info response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginServiceInfoResponse)]
pub struct PluginServiceInfoResponseTest {
    pub test_service_uuid: Uuid,
    pub test_characteristic_uuids: HeaplessVec<Uuid, TEST_MAX_CHARACTERISTICS>,
    pub test_service_exists: bool,
    pub test_service_active: bool,
    pub test_characteristic_count: u8,
}

/// Test plugin characteristic info response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginCharacteristicInfoResponse)]
pub struct PluginCharacteristicInfoResponseTest {
    pub test_char_uuid: Uuid,
    pub test_service_uuid: Uuid,
    pub test_properties: HeaplessVec<BLEPropertiesTest, TEST_MAX_PROPERTIES>,
    pub test_char_exists: bool,
    pub test_value_length: u8,
    pub test_client_config: u8,
}

/// Test plugin authentication completed response
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginAuthenticationCompletedResponse)]
pub struct PluginAuthenticationCompletedResponseTest {
    pub test_device_address: HeaplessVec<u8, 6>,
    pub test_address_type: BluetoothAddressTypeTest,
    pub test_auth_success: bool,
    pub test_auth_level: u8,
    pub test_bond_created: bool,
}

// ====== INTEGER TYPE REGRESSION TEST STRUCTURES ======

/// Small test struct with all integer types - fits in 64 bytes
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandGetServiceInfo)]
pub struct IntegerTypesTestSmall {
    pub test_u8: u8,
    pub test_u16: u16,
    pub test_u32: u32,
    pub test_i8: i8,
    pub test_i16: i16,
    pub test_i32: i32,
    pub test_f32: f32,
    pub test_f64: f64,
}

/// Test struct with U64/I64 types - designed to fit in 64 bytes
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[PluginIOMacro(MessageTypeId::PluginConfigurationError)]
pub struct IntegerTypesTestWithU64I64 {
    pub test_u64: u64,
    pub test_i64: i64,
    pub test_u16: u16,
    pub test_i16: i16,
    pub test_enabled: bool,
}

/// Mixed integer types test - compact for 64 bytes
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[HostIOMacro(MessageTypeId::HostCommandGetCharacteristicInfo)]
pub struct IntegerTypesTestMixed {
    pub test_u8: u8,
    pub test_i8: i8,
    pub test_u16: u16,
    pub test_i16: i16,
    pub test_f32: f32,
    pub test_bool: bool,
}

// ====== TEST DATA GENERATION FUNCTIONS ======

fn create_test_host_commands() -> Vec<(String, Vec<u8>)> {
    let mut test_data = Vec::new();

    // Test peripheral configuration command
    let test_peripheral_cmd = HostCommandConfigurePeripheralTest {
        test_name: HeaplessString::try_from("TestPeripheral").unwrap(),
        test_uuid: Uuid::parse_str("12345678-1234-5678-9abc-123456789abc").unwrap(),
        test_enabled: true,
        test_power_level: 4,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_peripheral_cmd.to_bytes().unwrap();
    test_data.push(("host_configure_peripheral".to_string(), serialized.to_vec()));

    // Test service configuration command
    let test_service_cmd = HostCommandConfigureServiceTest {
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_priority: 100,
        test_visible: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_service_cmd.to_bytes().unwrap();
    test_data.push(("host_configure_service".to_string(), serialized.to_vec()));

    // Test characteristic configuration command
    let mut properties = HeaplessVec::new();
    properties.push(BLEPropertiesTest::TestRead).unwrap();
    properties.push(BLEPropertiesTest::TestWrite).unwrap();
    properties.push(BLEPropertiesTest::TestNotify).unwrap();

    let test_characteristic_cmd = HostCommandConfigureCharacteristicTest {
        test_char_uuid: Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap(),
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_properties: properties,
        test_security_level: 2,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_characteristic_cmd.to_bytes().unwrap();
    test_data.push((
        "host_configure_characteristic".to_string(),
        serialized.to_vec(),
    ));

    // Test characteristic read configuration command
    let mut default_value = HeaplessVec::new();
    default_value
        .extend_from_slice(&[0x48, 0x65, 0x6c, 0x6c, 0x6f])
        .unwrap(); // "Hello"

    let test_char_read_cmd = HostCommandConfigureCharacteristicReadTest {
        test_char_uuid: Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap(),
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_default_value: default_value,
        test_read_permissions: 1,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_char_read_cmd.to_bytes().unwrap();
    test_data.push((
        "host_configure_characteristic_read".to_string(),
        serialized.to_vec(),
    ));

    // Test service info query command
    let test_service_info_cmd = HostCommandGetServiceInfoTest {
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_include_characteristics: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_service_info_cmd.to_bytes().unwrap();
    test_data.push(("host_get_service_info".to_string(), serialized.to_vec()));

    // Test characteristic info query command
    let test_char_info_cmd = HostCommandGetCharacteristicInfoTest {
        test_char_uuid: Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap(),
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_detailed_info: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_char_info_cmd.to_bytes().unwrap();
    test_data.push((
        "host_get_characteristic_info".to_string(),
        serialized.to_vec(),
    ));

    // Test advertisement start command
    let test_adv_cmd = HostCommandStartAdvertisementTest {
        test_allow_multi_connect: false,
        test_advertisement_interval: 100,
        test_tx_power: 50,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_adv_cmd.to_bytes().unwrap();
    test_data.push(("host_start_advertisement".to_string(), serialized.to_vec()));

    // Test notification command
    let mut notification_value = HeaplessVec::new();
    notification_value
        .extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05])
        .unwrap();

    let mut device_address = HeaplessVec::new();
    device_address
        .extend_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
        .unwrap();

    let test_notify_cmd = HostCommandNotifyCharacteristicValueTest {
        test_device_address: device_address,
        test_address_type: BluetoothAddressTypeTest::TestPublic,
        test_char_uuid: Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap(),
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_notification_value: notification_value,
        test_confirm_required: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_notify_cmd.to_bytes().unwrap();
    test_data.push((
        "host_notify_characteristic_value".to_string(),
        serialized.to_vec(),
    ));

    test_data
}

fn create_test_plugin_responses() -> Vec<(String, Vec<u8>)> {
    let mut test_data = Vec::new();

    // Test plugin data response
    let payload_data = b"Test data";
    let test_plugin_data = PluginDataTest {
        test_source_id: Uuid::parse_str("fedcba09-8765-4321-0fed-cba987654321").unwrap(),
        test_send_type: DataSendTypeTest::TestWrite,
        test_payload: payload_data,
        test_timestamp: 123,
        test_connection_handle: 1,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_plugin_data.to_bytes().unwrap();
    test_data.push(("plugin_data".to_string(), serialized.to_vec()));

    // Test plugin configuration error response
    let test_config_error = PluginConfigurationErrorTest {
        test_error_type: ConfigurationErrorTest::TestInvalidUuid,
        test_error_code: 41,
        test_error_description: HeaplessString::try_from("Invalid UUID").unwrap(),
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_config_error.to_bytes().unwrap();
    test_data.push((
        "plugin_configuration_error".to_string(),
        serialized.to_vec(),
    ));

    // Test plugin service info response
    let mut char_uuids = HeaplessVec::new();
    char_uuids
        .push(Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap())
        .unwrap();
    char_uuids
        .push(Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap())
        .unwrap();

    let test_service_info = PluginServiceInfoResponseTest {
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_characteristic_uuids: char_uuids,
        test_service_exists: true,
        test_service_active: true,
        test_characteristic_count: 2,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_service_info.to_bytes().unwrap();
    test_data.push((
        "plugin_service_info_response".to_string(),
        serialized.to_vec(),
    ));

    // Test plugin characteristic info response
    let mut char_properties = HeaplessVec::new();
    char_properties.push(BLEPropertiesTest::TestRead).unwrap();
    char_properties.push(BLEPropertiesTest::TestNotify).unwrap();

    let test_char_info = PluginCharacteristicInfoResponseTest {
        test_char_uuid: Uuid::parse_str("abcdef01-2345-6789-abcd-ef0123456789").unwrap(),
        test_service_uuid: Uuid::parse_str("87654321-4321-8765-cba9-987654321cba").unwrap(),
        test_properties: char_properties,
        test_char_exists: true,
        test_value_length: 20,
        test_client_config: 1,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_char_info.to_bytes().unwrap();
    test_data.push((
        "plugin_characteristic_info_response".to_string(),
        serialized.to_vec(),
    ));

    // Test plugin authentication completed response
    let mut auth_device_address = HeaplessVec::new();
    auth_device_address
        .extend_from_slice(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])
        .unwrap();

    let test_auth_completed = PluginAuthenticationCompletedResponseTest {
        test_device_address: auth_device_address,
        test_address_type: BluetoothAddressTypeTest::TestRandom,
        test_auth_success: true,
        test_auth_level: 3,
        test_bond_created: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = test_auth_completed.to_bytes().unwrap();
    test_data.push((
        "plugin_authentication_completed_response".to_string(),
        serialized.to_vec(),
    ));

    test_data
}

fn create_integer_type_test_data() -> Vec<(String, Vec<u8>)> {
    let mut test_data = Vec::new();

    // Test IntegerTypesTestSmall
    let small_test = IntegerTypesTestSmall {
        test_u8: 255,
        test_u16: 65535,
        test_u32: 4294967295,
        test_i8: -128,
        test_i16: -32768,
        test_i32: -2147483648,
        test_f32: 3.14159,
        test_f64: 2.718281828459045,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = small_test.to_bytes().unwrap();
    test_data.push(("integer_types_small".to_string(), serialized.to_vec()));

    // Test IntegerTypesTestWithU64I64
    let u64_i64_test = IntegerTypesTestWithU64I64 {
        test_u64: 18446744073709551615, // Max U64
        test_i64: -9223372036854775808, // Min I64
        test_u16: 12345,
        test_i16: -12345,
        test_enabled: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = u64_i64_test.to_bytes().unwrap();
    test_data.push(("integer_types_u64_i64".to_string(), serialized.to_vec()));

    // Test IntegerTypesTestMixed
    let mixed_test = IntegerTypesTestMixed {
        test_u8: 42,
        test_i8: -42,
        test_u16: 1234,
        test_i16: -1234,
        test_f32: 1.414213562373095, // sqrt(2)
        test_bool: true,
    };
    let serialized: [u8; DEFAULT_PACKET_SIZE] = mixed_test.to_bytes().unwrap();
    test_data.push(("integer_types_mixed".to_string(), serialized.to_vec()));

    test_data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating test data for protocol regression tests...");

    // Create output directory
    let output_dir = Path::new("../pc/python/python_regression_test_data/binary");
    fs::create_dir_all(output_dir)?;

    // Generate host command test data
    let host_commands = create_test_host_commands();
    for (name, data) in host_commands {
        let file_path = output_dir.join(format!("test_{}.bin", name));
        fs::write(&file_path, &data)?;
        println!("Generated: {} ({} bytes)", file_path.display(), data.len());
    }

    // Generate plugin response test data
    let plugin_responses = create_test_plugin_responses();
    for (name, data) in plugin_responses {
        let file_path = output_dir.join(format!("test_{}.bin", name));
        fs::write(&file_path, &data)?;
        println!("Generated: {} ({} bytes)", file_path.display(), data.len());
    }

    // Generate integer type test data
    let integer_tests = create_integer_type_test_data();
    for (name, data) in integer_tests {
        let file_path = output_dir.join(format!("test_{}.bin", name));
        fs::write(&file_path, &data)?;
        println!("Generated: {} ({} bytes)", file_path.display(), data.len());
    }

    println!("Test data generation completed successfully!");
    println!(
        "Generated {} host command files, {} plugin response files, and {} integer type test files",
        create_test_host_commands().len(),
        create_test_plugin_responses().len(),
        create_integer_type_test_data().len()
    );

    Ok(())
}
