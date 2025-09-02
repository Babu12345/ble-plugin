// Automatically generated rust module for 'protocol.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


#![allow(missing_docs)]
use crate::{IO, IOBase, HostIO, PluginIO, MessageType};
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(strum::EnumIter)]
pub enum MessageTypeId {
    MESSAGE_TYPE_ID_UNSPECIFIED = 0,
    TypeHostCommandConfigurePeripheral = 1,
    TypeHostCommandConfigureService = 2,
    TypeHostCommandConfigureCharacteristic = 3,
    TypeHostCommandConfigureCharacteristicRead = 4,
    TypeHostCommandGetServiceInfo = 5,
    TypeHostCommandGetCharacteristicInfo = 6,
    TypeHostCommandStartAdvertisement = 7,
    TypeHostCommandNotifyCharacteristicValue = 8,
    TypeHostCommandConfigurePeripheralSecurity = 9,
    TypeHostCommandConfigureProfile = 10,
    TypeHostCommandStopAdvertisement = 11,
    TypePluginData = 128,
    TypePluginConfigurationError = 129,
    TypePluginServiceInfoResponse = 130,
    TypePluginCharacteristicInfoResponse = 131,
    TypePluginAuthenticationCompletedResponse = 132,
}

impl Default for MessageTypeId {
    fn default() -> Self {
        MessageTypeId::MESSAGE_TYPE_ID_UNSPECIFIED
    }
}

impl From<i32> for MessageTypeId {
    fn from(i: i32) -> Self {
        match i {
            0 => MessageTypeId::MESSAGE_TYPE_ID_UNSPECIFIED,
            1 => MessageTypeId::TypeHostCommandConfigurePeripheral,
            2 => MessageTypeId::TypeHostCommandConfigureService,
            3 => MessageTypeId::TypeHostCommandConfigureCharacteristic,
            4 => MessageTypeId::TypeHostCommandConfigureCharacteristicRead,
            5 => MessageTypeId::TypeHostCommandGetServiceInfo,
            6 => MessageTypeId::TypeHostCommandGetCharacteristicInfo,
            7 => MessageTypeId::TypeHostCommandStartAdvertisement,
            8 => MessageTypeId::TypeHostCommandNotifyCharacteristicValue,
            9 => MessageTypeId::TypeHostCommandConfigurePeripheralSecurity,
            10 => MessageTypeId::TypeHostCommandConfigureProfile,
            11 => MessageTypeId::TypeHostCommandStopAdvertisement,
            128 => MessageTypeId::TypePluginData,
            129 => MessageTypeId::TypePluginConfigurationError,
            130 => MessageTypeId::TypePluginServiceInfoResponse,
            131 => MessageTypeId::TypePluginCharacteristicInfoResponse,
            132 => MessageTypeId::TypePluginAuthenticationCompletedResponse,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for MessageTypeId {
    fn from(s: &'a str) -> Self {
        match s {
            "MESSAGE_TYPE_ID_UNSPECIFIED" => MessageTypeId::MESSAGE_TYPE_ID_UNSPECIFIED,
            "TypeHostCommandConfigurePeripheral" => MessageTypeId::TypeHostCommandConfigurePeripheral,
            "TypeHostCommandConfigureService" => MessageTypeId::TypeHostCommandConfigureService,
            "TypeHostCommandConfigureCharacteristic" => MessageTypeId::TypeHostCommandConfigureCharacteristic,
            "TypeHostCommandConfigureCharacteristicRead" => MessageTypeId::TypeHostCommandConfigureCharacteristicRead,
            "TypeHostCommandGetServiceInfo" => MessageTypeId::TypeHostCommandGetServiceInfo,
            "TypeHostCommandGetCharacteristicInfo" => MessageTypeId::TypeHostCommandGetCharacteristicInfo,
            "TypeHostCommandStartAdvertisement" => MessageTypeId::TypeHostCommandStartAdvertisement,
            "TypeHostCommandNotifyCharacteristicValue" => MessageTypeId::TypeHostCommandNotifyCharacteristicValue,
            "TypeHostCommandConfigurePeripheralSecurity" => MessageTypeId::TypeHostCommandConfigurePeripheralSecurity,
            "TypeHostCommandConfigureProfile" => MessageTypeId::TypeHostCommandConfigureProfile,
            "TypeHostCommandStopAdvertisement" => MessageTypeId::TypeHostCommandStopAdvertisement,
            "TypePluginData" => MessageTypeId::TypePluginData,
            "TypePluginConfigurationError" => MessageTypeId::TypePluginConfigurationError,
            "TypePluginServiceInfoResponse" => MessageTypeId::TypePluginServiceInfoResponse,
            "TypePluginCharacteristicInfoResponse" => MessageTypeId::TypePluginCharacteristicInfoResponse,
            "TypePluginAuthenticationCompletedResponse" => MessageTypeId::TypePluginAuthenticationCompletedResponse,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum BLEProperties {
    BLE_PROPERTIES_UNSPECIFIED = 0,
    READ = 1,
    WRITE = 2,
    WRITE_NO_RSP = 3,
    NOTIFY = 4,
    INDICATE = 5,
}

impl Default for BLEProperties {
    fn default() -> Self {
        BLEProperties::BLE_PROPERTIES_UNSPECIFIED
    }
}

impl From<i32> for BLEProperties {
    fn from(i: i32) -> Self {
        match i {
            0 => BLEProperties::BLE_PROPERTIES_UNSPECIFIED,
            1 => BLEProperties::READ,
            2 => BLEProperties::WRITE,
            3 => BLEProperties::WRITE_NO_RSP,
            4 => BLEProperties::NOTIFY,
            5 => BLEProperties::INDICATE,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BLEProperties {
    fn from(s: &'a str) -> Self {
        match s {
            "BLE_PROPERTIES_UNSPECIFIED" => BLEProperties::BLE_PROPERTIES_UNSPECIFIED,
            "READ" => BLEProperties::READ,
            "WRITE" => BLEProperties::WRITE,
            "WRITE_NO_RSP" => BLEProperties::WRITE_NO_RSP,
            "NOTIFY" => BLEProperties::NOTIFY,
            "INDICATE" => BLEProperties::INDICATE,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum BluetoothAddressType {
    BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED = 0,
    PUBLIC = 1,
    RANDOM = 2,
    PUBLIC_ID = 3,
    RANDOM_ID = 4,
}

impl Default for BluetoothAddressType {
    fn default() -> Self {
        BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED
    }
}

impl From<i32> for BluetoothAddressType {
    fn from(i: i32) -> Self {
        match i {
            0 => BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED,
            1 => BluetoothAddressType::PUBLIC,
            2 => BluetoothAddressType::RANDOM,
            3 => BluetoothAddressType::PUBLIC_ID,
            4 => BluetoothAddressType::RANDOM_ID,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BluetoothAddressType {
    fn from(s: &'a str) -> Self {
        match s {
            "BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED" => BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED,
            "PUBLIC" => BluetoothAddressType::PUBLIC,
            "RANDOM" => BluetoothAddressType::RANDOM,
            "PUBLIC_ID" => BluetoothAddressType::PUBLIC_ID,
            "RANDOM_ID" => BluetoothAddressType::RANDOM_ID,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum BLEProfile {
    BLE_PROFILE_UNSPECIFIED = 0,
    CUSTOM = 1,
    HEART_RATE_MONITOR = 2,
    BATTERY_SERVICE = 3,
    DEVICE_INFORMATION = 4,
}

impl Default for BLEProfile {
    fn default() -> Self {
        BLEProfile::BLE_PROFILE_UNSPECIFIED
    }
}

impl From<i32> for BLEProfile {
    fn from(i: i32) -> Self {
        match i {
            0 => BLEProfile::BLE_PROFILE_UNSPECIFIED,
            1 => BLEProfile::CUSTOM,
            2 => BLEProfile::HEART_RATE_MONITOR,
            3 => BLEProfile::BATTERY_SERVICE,
            4 => BLEProfile::DEVICE_INFORMATION,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BLEProfile {
    fn from(s: &'a str) -> Self {
        match s {
            "BLE_PROFILE_UNSPECIFIED" => BLEProfile::BLE_PROFILE_UNSPECIFIED,
            "CUSTOM" => BLEProfile::CUSTOM,
            "HEART_RATE_MONITOR" => BLEProfile::HEART_RATE_MONITOR,
            "BATTERY_SERVICE" => BLEProfile::BATTERY_SERVICE,
            "DEVICE_INFORMATION" => BLEProfile::DEVICE_INFORMATION,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum PluginDataSendType {
    PLUGIN_DATA_SEND_TYPE_UNSPECIFIED = 0,
    NOTIFY_TYPE = 1,
    READ_TYPE = 2,
    WRITE_TYPE = 3,
}

impl Default for PluginDataSendType {
    fn default() -> Self {
        PluginDataSendType::PLUGIN_DATA_SEND_TYPE_UNSPECIFIED
    }
}

impl From<i32> for PluginDataSendType {
    fn from(i: i32) -> Self {
        match i {
            0 => PluginDataSendType::PLUGIN_DATA_SEND_TYPE_UNSPECIFIED,
            1 => PluginDataSendType::NOTIFY_TYPE,
            2 => PluginDataSendType::READ_TYPE,
            3 => PluginDataSendType::WRITE_TYPE,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PluginDataSendType {
    fn from(s: &'a str) -> Self {
        match s {
            "PLUGIN_DATA_SEND_TYPE_UNSPECIFIED" => PluginDataSendType::PLUGIN_DATA_SEND_TYPE_UNSPECIFIED,
            "NOTIFY_TYPE" => PluginDataSendType::NOTIFY_TYPE,
            "READ_TYPE" => PluginDataSendType::READ_TYPE,
            "WRITE_TYPE" => PluginDataSendType::WRITE_TYPE,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum PluginConfigurationErrorType {
    PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED = 0,
    PERIPHERAL_NAME_TOO_LONG = 1,
    INVALID_PERIPHERAL_UUID = 2,
    INVALID_SERVICE_UUID = 3,
    INVALID_CHARACTERISTIC_UUID = 4,
    ADVERTISEMENT_WITHOUT_PERIPHERAL_CONFIGURATION = 5,
    SERVICE_WITHOUT_PERIPHERAL_CONFIGURATION = 6,
    CHARACTERISTIC_WITHOUT_SERVICE_CONFIGURATION = 7,
}

impl Default for PluginConfigurationErrorType {
    fn default() -> Self {
        PluginConfigurationErrorType::PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED
    }
}

impl From<i32> for PluginConfigurationErrorType {
    fn from(i: i32) -> Self {
        match i {
            0 => PluginConfigurationErrorType::PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED,
            1 => PluginConfigurationErrorType::PERIPHERAL_NAME_TOO_LONG,
            2 => PluginConfigurationErrorType::INVALID_PERIPHERAL_UUID,
            3 => PluginConfigurationErrorType::INVALID_SERVICE_UUID,
            4 => PluginConfigurationErrorType::INVALID_CHARACTERISTIC_UUID,
            5 => PluginConfigurationErrorType::ADVERTISEMENT_WITHOUT_PERIPHERAL_CONFIGURATION,
            6 => PluginConfigurationErrorType::SERVICE_WITHOUT_PERIPHERAL_CONFIGURATION,
            7 => PluginConfigurationErrorType::CHARACTERISTIC_WITHOUT_SERVICE_CONFIGURATION,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PluginConfigurationErrorType {
    fn from(s: &'a str) -> Self {
        match s {
            "PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED" => PluginConfigurationErrorType::PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED,
            "PERIPHERAL_NAME_TOO_LONG" => PluginConfigurationErrorType::PERIPHERAL_NAME_TOO_LONG,
            "INVALID_PERIPHERAL_UUID" => PluginConfigurationErrorType::INVALID_PERIPHERAL_UUID,
            "INVALID_SERVICE_UUID" => PluginConfigurationErrorType::INVALID_SERVICE_UUID,
            "INVALID_CHARACTERISTIC_UUID" => PluginConfigurationErrorType::INVALID_CHARACTERISTIC_UUID,
            "ADVERTISEMENT_WITHOUT_PERIPHERAL_CONFIGURATION" => PluginConfigurationErrorType::ADVERTISEMENT_WITHOUT_PERIPHERAL_CONFIGURATION,
            "SERVICE_WITHOUT_PERIPHERAL_CONFIGURATION" => PluginConfigurationErrorType::SERVICE_WITHOUT_PERIPHERAL_CONFIGURATION,
            "CHARACTERISTIC_WITHOUT_SERVICE_CONFIGURATION" => PluginConfigurationErrorType::CHARACTERISTIC_WITHOUT_SERVICE_CONFIGURATION,
            _ => Self::default(),
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigurePeripheral)]
pub struct HostCommandConfigurePeripheral {
    pub name: String,
    pub addr: Vec<u8>,
}

impl<'a> MessageRead<'a> for HostCommandConfigurePeripheral {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.name = r.read_string(bytes)?.to_owned(),
                Ok(18) => msg.addr = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigurePeripheral {
    fn get_size(&self) -> usize {
        0
        + if self.name == String::default() { 0 } else { 1 + sizeof_len((&self.name).len()) }
        + if self.addr.is_empty() { 0 } else { 1 + sizeof_len((&self.addr).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.name != String::default() { w.write_with_tag(10, |w| w.write_string(&**&self.name))?; }
        if !self.addr.is_empty() { w.write_with_tag(18, |w| w.write_bytes(&**&self.addr))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigurePeripheral)]
pub struct HostCommandConfigurePeripheralSecurity {
    pub passkey: u32,
}

impl<'a> MessageRead<'a> for HostCommandConfigurePeripheralSecurity {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.passkey = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigurePeripheralSecurity {
    fn get_size(&self) -> usize {
        0
        + if self.passkey == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.passkey) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.passkey != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.passkey))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigureService)]
pub struct HostCommandConfigureService {
    pub uuid: u32,
}

impl<'a> MessageRead<'a> for HostCommandConfigureService {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.uuid = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigureService {
    fn get_size(&self) -> usize {
        0
        + if self.uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.uuid) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.uuid))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigureCharacteristic)]
pub struct HostCommandConfigureCharacteristic {
    pub uuid: u32,
    pub service_uuid: u32,
    pub properties: Vec<protocol::BLEProperties>,
}

impl<'a> MessageRead<'a> for HostCommandConfigureCharacteristic {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.uuid = r.read_uint32(bytes)?,
                Ok(16) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(26) => msg.properties = r.read_packed(bytes, |r, bytes| Ok(r.read_enum(bytes)?))?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigureCharacteristic {
    fn get_size(&self) -> usize {
        0
        + if self.uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.properties.is_empty() { 0 } else { 1 + sizeof_len(self.properties.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.service_uuid))?; }
        w.write_packed_with_tag(26, &self.properties, |w, m| w.write_enum(*m as i32), &|m| sizeof_varint(*(m) as u64))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigureCharacteristic)]
pub struct HostCommandConfigureCharacteristicRead {
    pub uuid: u32,
    pub service_uuid: u32,
    pub value: Vec<u8>,
}

impl<'a> MessageRead<'a> for HostCommandConfigureCharacteristicRead {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.uuid = r.read_uint32(bytes)?,
                Ok(16) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(26) => msg.value = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigureCharacteristicRead {
    fn get_size(&self) -> usize {
        0
        + if self.uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.value.is_empty() { 0 } else { 1 + sizeof_len((&self.value).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.service_uuid))?; }
        if !self.value.is_empty() { w.write_with_tag(26, |w| w.write_bytes(&**&self.value))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandGetServiceInfo)]
pub struct HostCommandGetServiceInfo {
    pub uuid: u32,
}

impl<'a> MessageRead<'a> for HostCommandGetServiceInfo {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.uuid = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandGetServiceInfo {
    fn get_size(&self) -> usize {
        0
        + if self.uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.uuid) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.uuid))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandGetCharacteristicInfo)]
pub struct HostCommandGetCharacteristicInfo {
    pub characteristic_uuid: u32,
    pub service_uuid: u32,
}

impl<'a> MessageRead<'a> for HostCommandGetCharacteristicInfo {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.characteristic_uuid = r.read_uint32(bytes)?,
                Ok(16) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandGetCharacteristicInfo {
    fn get_size(&self) -> usize {
        0
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.characteristic_uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.characteristic_uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.service_uuid))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandStartAdvertisement)]
pub struct HostCommandStartAdvertisement {
    pub allow_multi_connect: bool,
}

impl<'a> MessageRead<'a> for HostCommandStartAdvertisement {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.allow_multi_connect = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandStartAdvertisement {
    fn get_size(&self) -> usize {
        0
        + if self.allow_multi_connect == false { 0 } else { 1 + sizeof_varint(*(&self.allow_multi_connect) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.allow_multi_connect != false { w.write_with_tag(8, |w| w.write_bool(*&self.allow_multi_connect))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandStopAdvertisement)]
pub struct HostCommandStopAdvertisement { }

impl<'a> MessageRead<'a> for HostCommandStopAdvertisement {
    fn from_reader(r: &mut BytesReader, _: &[u8]) -> Result<Self> {
        r.read_to_end();
        Ok(Self::default())
    }
}

impl MessageWrite for HostCommandStopAdvertisement { }

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandNotifyCharacteristicValue)]
pub struct HostCommandNotifyCharacteristicValue {
    pub address: Vec<u8>,
    pub address_type: protocol::BluetoothAddressType,
    pub characteristic_uuid: u32,
    pub service_uuid: u32,
    pub value: Vec<u8>,
}

impl<'a> MessageRead<'a> for HostCommandNotifyCharacteristicValue {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.address = r.read_bytes(bytes)?.to_owned(),
                Ok(16) => msg.address_type = r.read_enum(bytes)?,
                Ok(24) => msg.characteristic_uuid = r.read_uint32(bytes)?,
                Ok(32) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(42) => msg.value = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandNotifyCharacteristicValue {
    fn get_size(&self) -> usize {
        0
        + if self.address.is_empty() { 0 } else { 1 + sizeof_len((&self.address).len()) }
        + if self.address_type == protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.address_type) as u64) }
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.value.is_empty() { 0 } else { 1 + sizeof_len((&self.value).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.address.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.address))?; }
        if self.address_type != protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { w.write_with_tag(16, |w| w.write_enum(*&self.address_type as i32))?; }
        if self.characteristic_uuid != 0u32 { w.write_with_tag(24, |w| w.write_uint32(*&self.characteristic_uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(32, |w| w.write_uint32(*&self.service_uuid))?; }
        if !self.value.is_empty() { w.write_with_tag(42, |w| w.write_bytes(&**&self.value))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigureProfile)]
pub struct HostCommandConfigureProfile {
    pub profile: protocol::BLEProfile,
}

impl<'a> MessageRead<'a> for HostCommandConfigureProfile {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.profile = r.read_enum(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for HostCommandConfigureProfile {
    fn get_size(&self) -> usize {
        0
        + if self.profile == protocol::BLEProfile::BLE_PROFILE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.profile) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.profile != protocol::BLEProfile::BLE_PROFILE_UNSPECIFIED { w.write_with_tag(8, |w| w.write_enum(*&self.profile as i32))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::PluginIO(MessageTypeId::TypePluginData)]
pub struct PluginData {
    pub src_addr: Vec<u8>,
    pub src_addr_type: protocol::BluetoothAddressType,
    pub send_type: protocol::PluginDataSendType,
    pub characteristic_uuid: u32,
    pub service_uuid: u32,
    pub data: Vec<u8>,
}

impl<'a> MessageRead<'a> for PluginData {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.src_addr = r.read_bytes(bytes)?.to_owned(),
                Ok(16) => msg.src_addr_type = r.read_enum(bytes)?,
                Ok(24) => msg.send_type = r.read_enum(bytes)?,
                Ok(32) => msg.characteristic_uuid = r.read_uint32(bytes)?,
                Ok(40) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(50) => msg.data = r.read_bytes(bytes)?.to_owned(),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PluginData {
    fn get_size(&self) -> usize {
        0
        + if self.src_addr.is_empty() { 0 } else { 1 + sizeof_len((&self.src_addr).len()) }
        + if self.src_addr_type == protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.src_addr_type) as u64) }
        + if self.send_type == protocol::PluginDataSendType::PLUGIN_DATA_SEND_TYPE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.send_type) as u64) }
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.data.is_empty() { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.src_addr.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.src_addr))?; }
        if self.src_addr_type != protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { w.write_with_tag(16, |w| w.write_enum(*&self.src_addr_type as i32))?; }
        if self.send_type != protocol::PluginDataSendType::PLUGIN_DATA_SEND_TYPE_UNSPECIFIED { w.write_with_tag(24, |w| w.write_enum(*&self.send_type as i32))?; }
        if self.characteristic_uuid != 0u32 { w.write_with_tag(32, |w| w.write_uint32(*&self.characteristic_uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(40, |w| w.write_uint32(*&self.service_uuid))?; }
        if !self.data.is_empty() { w.write_with_tag(50, |w| w.write_bytes(&**&self.data))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::PluginIO(MessageTypeId::TypePluginConfigurationError)]
pub struct PluginConfigurationError {
    pub error_type: protocol::PluginConfigurationErrorType,
}

impl<'a> MessageRead<'a> for PluginConfigurationError {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.error_type = r.read_enum(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PluginConfigurationError {
    fn get_size(&self) -> usize {
        0
        + if self.error_type == protocol::PluginConfigurationErrorType::PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.error_type) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.error_type != protocol::PluginConfigurationErrorType::PLUGIN_CONFIGURATION_ERROR_TYPE_UNSPECIFIED { w.write_with_tag(8, |w| w.write_enum(*&self.error_type as i32))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::PluginIO(MessageTypeId::TypePluginServiceInfoResponse)]
pub struct PluginServiceInfoResponse {
    pub service_uuid: u32,
    pub characteristic_uuids: Vec<u32>,
    pub exists: bool,
}

impl<'a> MessageRead<'a> for PluginServiceInfoResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(18) => msg.characteristic_uuids = r.read_packed(bytes, |r, bytes| Ok(r.read_uint32(bytes)?))?,
                Ok(24) => msg.exists = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PluginServiceInfoResponse {
    fn get_size(&self) -> usize {
        0
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.characteristic_uuids.is_empty() { 0 } else { 1 + sizeof_len(self.characteristic_uuids.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
        + if self.exists == false { 0 } else { 1 + sizeof_varint(*(&self.exists) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.service_uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.service_uuid))?; }
        w.write_packed_with_tag(18, &self.characteristic_uuids, |w, m| w.write_uint32(*m), &|m| sizeof_varint(*(m) as u64))?;
        if self.exists != false { w.write_with_tag(24, |w| w.write_bool(*&self.exists))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::PluginIO(MessageTypeId::TypePluginCharacteristicInfoResponse)]
pub struct PluginCharacteristicInfoResponse {
    pub characteristic_uuid: u32,
    pub service_uuid: u32,
    pub properties: Vec<protocol::BLEProperties>,
    pub exists: bool,
}

impl<'a> MessageRead<'a> for PluginCharacteristicInfoResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.characteristic_uuid = r.read_uint32(bytes)?,
                Ok(16) => msg.service_uuid = r.read_uint32(bytes)?,
                Ok(26) => msg.properties = r.read_packed(bytes, |r, bytes| Ok(r.read_enum(bytes)?))?,
                Ok(32) => msg.exists = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PluginCharacteristicInfoResponse {
    fn get_size(&self) -> usize {
        0
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.properties.is_empty() { 0 } else { 1 + sizeof_len(self.properties.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
        + if self.exists == false { 0 } else { 1 + sizeof_varint(*(&self.exists) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.characteristic_uuid != 0u32 { w.write_with_tag(8, |w| w.write_uint32(*&self.characteristic_uuid))?; }
        if self.service_uuid != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.service_uuid))?; }
        w.write_packed_with_tag(26, &self.properties, |w, m| w.write_enum(*m as i32), &|m| sizeof_varint(*(m) as u64))?;
        if self.exists != false { w.write_with_tag(32, |w| w.write_bool(*&self.exists))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
#[protocol_io::PluginIO(MessageTypeId::TypePluginAuthenticationCompletedResponse)]
pub struct PluginAuthenticationCompletedResponse {
    pub address: Vec<u8>,
    pub address_type: protocol::BluetoothAddressType,
    pub success: bool,
}

impl<'a> MessageRead<'a> for PluginAuthenticationCompletedResponse {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.address = r.read_bytes(bytes)?.to_owned(),
                Ok(16) => msg.address_type = r.read_enum(bytes)?,
                Ok(24) => msg.success = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for PluginAuthenticationCompletedResponse {
    fn get_size(&self) -> usize {
        0
        + if self.address.is_empty() { 0 } else { 1 + sizeof_len((&self.address).len()) }
        + if self.address_type == protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { 0 } else { 1 + sizeof_varint(*(&self.address_type) as u64) }
        + if self.success == false { 0 } else { 1 + sizeof_varint(*(&self.success) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.address.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.address))?; }
        if self.address_type != protocol::BluetoothAddressType::BLUETOOTH_ADDRESS_TYPE_UNSPECIFIED { w.write_with_tag(16, |w| w.write_enum(*&self.address_type as i32))?; }
        if self.success != false { w.write_with_tag(24, |w| w.write_bool(*&self.success))?; }
        Ok(())
    }
}

