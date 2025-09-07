// Automatically generated rust module for 'protocol.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


#![allow(missing_docs)]

extern crate alloc;
use crate::{IO, IOBase, HostIO, PluginIO, MessageType};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::borrow::ToOwned;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageTypeId {
    MessageTypeIdUnspecified = 0,
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
        MessageTypeId::MessageTypeIdUnspecified
    }
}

impl From<i32> for MessageTypeId {
    fn from(i: i32) -> Self {
        match i {
            0 => MessageTypeId::MessageTypeIdUnspecified,
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
            "MessageTypeIdUnspecified" => MessageTypeId::MessageTypeIdUnspecified,
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
pub enum BleProperties {
    BlePropertiesUnspecified = 0,
    Read = 1,
    WriteRsp = 2,
    WriteNoRsp = 3,
    Notify = 4,
    Indicate = 5,
}

impl Default for BleProperties {
    fn default() -> Self {
        BleProperties::BlePropertiesUnspecified
    }
}

impl From<i32> for BleProperties {
    fn from(i: i32) -> Self {
        match i {
            0 => BleProperties::BlePropertiesUnspecified,
            1 => BleProperties::Read,
            2 => BleProperties::WriteRsp,
            3 => BleProperties::WriteNoRsp,
            4 => BleProperties::Notify,
            5 => BleProperties::Indicate,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BleProperties {
    fn from(s: &'a str) -> Self {
        match s {
            "BlePropertiesUnspecified" => BleProperties::BlePropertiesUnspecified,
            "Read" => BleProperties::Read,
            "WriteRsp" => BleProperties::WriteRsp,
            "WriteNoRsp" => BleProperties::WriteNoRsp,
            "Notify" => BleProperties::Notify,
            "Indicate" => BleProperties::Indicate,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum BluetoothAddressType {
    BluetoothAddressTypeUnspecified = 0,
    Public = 1,
    Random = 2,
    PublicId = 3,
    RandomId = 4,
}

impl Default for BluetoothAddressType {
    fn default() -> Self {
        BluetoothAddressType::BluetoothAddressTypeUnspecified
    }
}

impl From<i32> for BluetoothAddressType {
    fn from(i: i32) -> Self {
        match i {
            0 => BluetoothAddressType::BluetoothAddressTypeUnspecified,
            1 => BluetoothAddressType::Public,
            2 => BluetoothAddressType::Random,
            3 => BluetoothAddressType::PublicId,
            4 => BluetoothAddressType::RandomId,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BluetoothAddressType {
    fn from(s: &'a str) -> Self {
        match s {
            "BluetoothAddressTypeUnspecified" => BluetoothAddressType::BluetoothAddressTypeUnspecified,
            "Public" => BluetoothAddressType::Public,
            "Random" => BluetoothAddressType::Random,
            "PublicId" => BluetoothAddressType::PublicId,
            "RandomId" => BluetoothAddressType::RandomId,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum BleProfile {
    BleProfileUnspecified = 0,
    Custom = 1,
    HeartRateMonitor = 2,
    BatteryService = 3,
    DeviceInformation = 4,
}

impl Default for BleProfile {
    fn default() -> Self {
        BleProfile::BleProfileUnspecified
    }
}

impl From<i32> for BleProfile {
    fn from(i: i32) -> Self {
        match i {
            0 => BleProfile::BleProfileUnspecified,
            1 => BleProfile::Custom,
            2 => BleProfile::HeartRateMonitor,
            3 => BleProfile::BatteryService,
            4 => BleProfile::DeviceInformation,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for BleProfile {
    fn from(s: &'a str) -> Self {
        match s {
            "BleProfileUnspecified" => BleProfile::BleProfileUnspecified,
            "Custom" => BleProfile::Custom,
            "HeartRateMonitor" => BleProfile::HeartRateMonitor,
            "BatteryService" => BleProfile::BatteryService,
            "DeviceInformation" => BleProfile::DeviceInformation,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum PluginDataSendType {
    PluginDataSendTypeUnspecified = 0,
    NotifyType = 1,
    ReadType = 2,
    WriteType = 3,
}

impl Default for PluginDataSendType {
    fn default() -> Self {
        PluginDataSendType::PluginDataSendTypeUnspecified
    }
}

impl From<i32> for PluginDataSendType {
    fn from(i: i32) -> Self {
        match i {
            0 => PluginDataSendType::PluginDataSendTypeUnspecified,
            1 => PluginDataSendType::NotifyType,
            2 => PluginDataSendType::ReadType,
            3 => PluginDataSendType::WriteType,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PluginDataSendType {
    fn from(s: &'a str) -> Self {
        match s {
            "PluginDataSendTypeUnspecified" => PluginDataSendType::PluginDataSendTypeUnspecified,
            "NotifyType" => PluginDataSendType::NotifyType,
            "ReadType" => PluginDataSendType::ReadType,
            "WriteType" => PluginDataSendType::WriteType,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum PluginConfigurationErrorType {
    PluginConfigurationErrorTypeUnspecified = 0,
    PeripheralNameTooLong = 1,
    InvalidPeripheralUuid = 2,
    InvalidServiceUuid = 3,
    InvalidCharacteristicUuid = 4,
    AdvertisementWithoutPeripheralConfiguration = 5,
    ServiceWithoutPeripheralConfiguration = 6,
    CharacteristicWithoutServiceConfiguration = 7,
}

impl Default for PluginConfigurationErrorType {
    fn default() -> Self {
        PluginConfigurationErrorType::PluginConfigurationErrorTypeUnspecified
    }
}

impl From<i32> for PluginConfigurationErrorType {
    fn from(i: i32) -> Self {
        match i {
            0 => PluginConfigurationErrorType::PluginConfigurationErrorTypeUnspecified,
            1 => PluginConfigurationErrorType::PeripheralNameTooLong,
            2 => PluginConfigurationErrorType::InvalidPeripheralUuid,
            3 => PluginConfigurationErrorType::InvalidServiceUuid,
            4 => PluginConfigurationErrorType::InvalidCharacteristicUuid,
            5 => PluginConfigurationErrorType::AdvertisementWithoutPeripheralConfiguration,
            6 => PluginConfigurationErrorType::ServiceWithoutPeripheralConfiguration,
            7 => PluginConfigurationErrorType::CharacteristicWithoutServiceConfiguration,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PluginConfigurationErrorType {
    fn from(s: &'a str) -> Self {
        match s {
            "PluginConfigurationErrorTypeUnspecified" => PluginConfigurationErrorType::PluginConfigurationErrorTypeUnspecified,
            "PeripheralNameTooLong" => PluginConfigurationErrorType::PeripheralNameTooLong,
            "InvalidPeripheralUuid" => PluginConfigurationErrorType::InvalidPeripheralUuid,
            "InvalidServiceUuid" => PluginConfigurationErrorType::InvalidServiceUuid,
            "InvalidCharacteristicUuid" => PluginConfigurationErrorType::InvalidCharacteristicUuid,
            "AdvertisementWithoutPeripheralConfiguration" => PluginConfigurationErrorType::AdvertisementWithoutPeripheralConfiguration,
            "ServiceWithoutPeripheralConfiguration" => PluginConfigurationErrorType::ServiceWithoutPeripheralConfiguration,
            "CharacteristicWithoutServiceConfiguration" => PluginConfigurationErrorType::CharacteristicWithoutServiceConfiguration,
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
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigurePeripheralSecurity)]
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
    pub properties: Vec<protocol::BleProperties>,
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
#[protocol_io::HostIO(MessageTypeId::TypeHostCommandConfigureCharacteristicRead)]
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
        + if self.address_type == protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { 0 } else { 1 + sizeof_varint(*(&self.address_type) as u64) }
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.value.is_empty() { 0 } else { 1 + sizeof_len((&self.value).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.address.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.address))?; }
        if self.address_type != protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { w.write_with_tag(16, |w| w.write_enum(*&self.address_type as i32))?; }
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
    pub profile: protocol::BleProfile,
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
        + if self.profile == protocol::BleProfile::BleProfileUnspecified { 0 } else { 1 + sizeof_varint(*(&self.profile) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.profile != protocol::BleProfile::BleProfileUnspecified { w.write_with_tag(8, |w| w.write_enum(*&self.profile as i32))?; }
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
        + if self.src_addr_type == protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { 0 } else { 1 + sizeof_varint(*(&self.src_addr_type) as u64) }
        + if self.send_type == protocol::PluginDataSendType::PluginDataSendTypeUnspecified { 0 } else { 1 + sizeof_varint(*(&self.send_type) as u64) }
        + if self.characteristic_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.characteristic_uuid) as u64) }
        + if self.service_uuid == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.service_uuid) as u64) }
        + if self.data.is_empty() { 0 } else { 1 + sizeof_len((&self.data).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.src_addr.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.src_addr))?; }
        if self.src_addr_type != protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { w.write_with_tag(16, |w| w.write_enum(*&self.src_addr_type as i32))?; }
        if self.send_type != protocol::PluginDataSendType::PluginDataSendTypeUnspecified { w.write_with_tag(24, |w| w.write_enum(*&self.send_type as i32))?; }
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
        + if self.error_type == protocol::PluginConfigurationErrorType::PluginConfigurationErrorTypeUnspecified { 0 } else { 1 + sizeof_varint(*(&self.error_type) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.error_type != protocol::PluginConfigurationErrorType::PluginConfigurationErrorTypeUnspecified { w.write_with_tag(8, |w| w.write_enum(*&self.error_type as i32))?; }
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
    pub properties: Vec<protocol::BleProperties>,
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
        + if self.address_type == protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { 0 } else { 1 + sizeof_varint(*(&self.address_type) as u64) }
        + if self.success == false { 0 } else { 1 + sizeof_varint(*(&self.success) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.address.is_empty() { w.write_with_tag(10, |w| w.write_bytes(&**&self.address))?; }
        if self.address_type != protocol::BluetoothAddressType::BluetoothAddressTypeUnspecified { w.write_with_tag(16, |w| w.write_enum(*&self.address_type as i32))?; }
        if self.success != false { w.write_with_tag(24, |w| w.write_bool(*&self.success))?; }
        Ok(())
    }
}

