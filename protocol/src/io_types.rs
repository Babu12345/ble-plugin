//! Contains the basic types to reuse

use serde::{Deserialize, Serialize};

pub use host::*;
pub use plugin::*;

/// Host types
pub mod host {
    use crate::HostIO;
    use crate::IO;
    use crate::MAX_NAME_SIZE;
    use crate::{MessageType, MessageTypeId};
    use heapless::String;
    use heapless::Vec;
    use protocol_io::HostIO;
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    /// Host command. Configure peripheral
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
    pub struct HostCommandConfigurePeripheral {
        /// Peripheral name
        pub name: String<MAX_NAME_SIZE>,
        /// Peripheral addr
        pub addr: [u8; 6],
    }

    /// Host command. Configure peripheral
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheralSecurity)]
    pub struct HostCommandConfigurePeripheralSecurity {
        /// Passkey for pairing (6 digit numeric)
        pub passkey: u32,
    }

    /// Host command. Configure service
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigureService)]
    pub struct HostCommandConfigureService {
        /// Service UUID
        pub uuid: Uuid,
    }

    /// Properties enumeration for BLE characteristics.
    #[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub enum BLEProperties {
        /// Read property
        READ = 0,
        /// Write property
        WRITE = 1,
        /// Write without response property
        WriteNoRsp = 2,
        /// Notify property
        NOTIFY = 3,
        /// Indicate property
        INDICATE = 4,
    }

    /// Maximum size for characteristic properties
    pub const MAX_PROPERTIES: usize = 4;

    /// Host command. Configure characteristic
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigureCharacteristic)]
    pub struct HostCommandConfigureCharacteristic {
        /// Characteristic UUID
        pub uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Properties of the characteristic (read, write, notify, etc.)
        pub properties: heapless::Vec<BLEProperties, MAX_PROPERTIES>, // Assuming max 4 properties per characteristic
    }

    /// Host command. Configure characteristic read
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigureCharacteristicRead)]
    pub struct HostCommandConfigureCharacteristicRead {
        /// Characteristic UUID
        pub uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Read value
        pub value: Vec<u8, MAX_NAME_SIZE>,
    }

    /// Host command. Get service info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandGetServiceInfo)]
    pub struct HostCommandGetServiceInfo {
        /// Service UUID
        pub uuid: Uuid,
    }

    /// Host command. Get characteristic info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandGetCharacteristicInfo)]
    pub struct HostCommandGetCharacteristicInfo {
        /// Characteristic UUID
        pub characteristic_uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
    }

    /// Host command. Start advertisement
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
    pub struct HostCommandStartAdvertisement {
        /// Allow multiple central connections
        pub allow_multi_connect: bool,
    }

    /// Bluetooth Device address type
    #[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
    #[repr(u8)]
    pub enum BluetoothAddressType {
        /// Public address
        Public = 0,
        /// Random address
        Random = 1,
        /// Public ID address
        PublicID = 2,
        /// Random ID address
        RandomID = 3,
    }

    /// Host command. Notify characteristic value
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandNotifyCharacteristicValue)]
    pub struct HostCommandNotifyCharacteristicValue {
        /// Device Address.
        pub address: [u8; 6],
        /// Address type
        pub address_type: BluetoothAddressType,
        /// Characteristic UUID
        pub characteristic_uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Value to notify
        pub value: Vec<u8, MAX_NAME_SIZE>,
    }
}

/// Plugin types
pub mod plugin {
    use crate::PluginIO;
    use crate::{MessageType, MessageTypeId};
    use protocol_io::PluginIO;
    use uuid::Uuid;

    use super::*;
    use crate::IO;

    /// Represents the send type of the data. Was it due to a
    /// write event (central -> peripheral), notify event (peripheral -> central),
    /// or read attempt (central -> peripheral). Depending on which, a response
    /// might be expected or sent
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[repr(u8)]
    pub enum PluginDataSendType {
        /// Notified from the central bluetooth device
        Notify = 0,
        /// Read attempt from the central bluetooth device
        Read = 1,
        /// Written from the central bluetooth device
        Write = 2,
    }

    /// Plugin data
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginData)]
    pub struct PluginData<'a> {
        /// Source peripheral id that this data is orginating from.
        pub src_id: Uuid,
        /// Send type of the data
        pub send_type: PluginDataSendType,
        /// Actual command type
        pub data: &'a [u8],
    }

    /// Represents the error that can occur during plugin configuration
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginConfigurationError)]
    #[repr(u8)]
    pub enum PluginConfigurationError {
        /// The peripheral name is too long
        PeripheralNameTooLong = 0,
        /// The peripheral UUID is invalid
        InvalidPeripheralUuid = 1,
        /// The service UUID is invalid
        InvalidServiceUuid = 2,
        /// The characteristic UUID is invalid
        InvalidCharacteristicUuid = 3,
        /// Advertisement without proper peripheral configuration
        AdvertisementWithoutPeripheralConfiguration = 4,
        /// Service without proper peripheral configuration
        ServiceWithoutPeripheralConfiguration = 5,
        /// Characteristic without proper service configuration
        CharacteristicWithoutServiceConfiguration = 6,
    }

    /// Maximum characteritics per service
    pub const MAX_CHARACTERISTICS_PER_SERVICE: usize = 16;

    /// Service information response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginServiceInfoResponse)]
    pub struct PluginServiceInfoResponse {
        /// Service UUID
        pub service_uuid: Uuid,
        /// List of characteristic UUIDs in this service
        pub characteristic_uuids: heapless::Vec<Uuid, MAX_CHARACTERISTICS_PER_SERVICE>, // Assuming max 16 characteristics per service
        /// Whether the service exists
        pub exists: bool,
    }

    /// Characteristic information response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginCharacteristicInfoResponse)]
    pub struct PluginCharacteristicInfoResponse {
        /// Characteristic UUID
        pub characteristic_uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Properties of the characteristic (read, write, notify, etc.)
        pub properties: heapless::Vec<BLEProperties, 4>,
        /// Whether the characteristic exists
        pub exists: bool,
    }

    /// Plugin authentication completed response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginAuthenticationCompletedResponse)]
    pub struct PluginAuthenticationCompletedResponse {
        /// Address of the device that was authenticated
        pub address: [u8; 6],
        /// Address type
        pub address_type: BluetoothAddressType,
        /// Whether the authentication was successful
        pub success: bool,
    }
}
