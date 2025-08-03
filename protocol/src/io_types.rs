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
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigurePeripheral {
        /// Peripheral name
        pub name: String<MAX_NAME_SIZE>,
        /// Peripheral UUID
        pub uuid: Uuid,
    }

    impl MessageType for HostCommandConfigurePeripheral {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandConfigurePeripheral
        }
    }

    /// Host command. Configure service
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureService {
        /// Service UUID
        pub uuid: Uuid,
    }

    impl MessageType for HostCommandConfigureService {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandConfigureService
        }
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
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureCharacteristic {
        /// Characteristic UUID
        pub uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Properties of the characteristic (read, write, notify, etc.)
        pub properties: heapless::Vec<BLEProperties, MAX_PROPERTIES>, // Assuming max 4 properties per characteristic
    }

    impl MessageType for HostCommandConfigureCharacteristic {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandConfigureCharacteristic
        }
    }

    /// Host command. Configure characteristic read
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureCharacteristicRead {
        /// Characteristic UUID
        pub uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
        /// Read value
        pub value: Vec<u8, MAX_NAME_SIZE>,
    }

    impl MessageType for HostCommandConfigureCharacteristicRead {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandConfigureCharacteristicRead
        }
    }

    /// Host command. Get service info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandGetServiceInfo {
        /// Service UUID
        pub uuid: Uuid,
    }

    impl MessageType for HostCommandGetServiceInfo {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandGetServiceInfo
        }
    }

    /// Host command. Get characteristic info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandGetCharacteristicInfo {
        /// Characteristic UUID
        pub characteristic_uuid: Uuid,
        /// Service UUID this characteristic belongs to
        pub service_uuid: Uuid,
    }

    impl MessageType for HostCommandGetCharacteristicInfo {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandGetCharacteristicInfo
        }
    }

    /// Host command. Start advertisement
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandStartAdvertisement {
        /// Allow multiple central connections
        pub allow_multi_connect: bool,
    }

    impl MessageType for HostCommandStartAdvertisement {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandStartAdvertisement
        }
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
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
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

    impl MessageType for HostCommandNotifyCharacteristicValue {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::HostCommandNotifyCharacteristicValue
        }
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
        Notify,
        /// Read attempt from the central bluetooth device
        Read,
        /// Written from the central bluetooth device
        Write,
    }

    /// Plugin data
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PluginIO)]
    pub struct PluginData<'a> {
        /// Source peripheral id that this data is orginating from.
        pub src_id: Uuid,
        /// Send type of the data
        pub send_type: PluginDataSendType,
        /// Actual command type
        pub data: &'a [u8],
    }

    impl<'a> MessageType for PluginData<'a> {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::PluginData
        }
    }

    /// Represents the error that can occur during plugin configuration
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PluginIO)]
    #[repr(u8)]
    pub enum PluginConfigurationError {
        /// The peripheral name is too long
        PeripheralNameTooLong,
        /// The peripheral UUID is invalid
        InvalidPeripheralUuid,
        /// The service UUID is invalid
        InvalidServiceUuid,
        /// The characteristic UUID is invalid
        InvalidCharacteristicUuid,
        /// Advertisement without proper peripheral configuration
        AdvertisementWithoutPeripheralConfiguration,
        /// Service without proper peripheral configuration
        ServiceWithoutPeripheralConfiguration,
        /// Characteristic without proper service configuration
        CharacteristicWithoutServiceConfiguration,
    }

    impl MessageType for PluginConfigurationError {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::PluginConfigurationError
        }
    }

    /// Maximum characteritics per service
    pub const MAX_CHARACTERISTICS_PER_SERVICE: usize = 16;

    /// Service information response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PluginIO)]
    pub struct PluginServiceInfoResponse {
        /// Service UUID
        pub service_uuid: Uuid,
        /// List of characteristic UUIDs in this service
        pub characteristic_uuids: heapless::Vec<Uuid, MAX_CHARACTERISTICS_PER_SERVICE>, // Assuming max 16 characteristics per service
        /// Whether the service exists
        pub exists: bool,
    }

    impl MessageType for PluginServiceInfoResponse {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::PluginServiceInfoResponse
        }
    }

    /// Characteristic information response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, PluginIO)]
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

    impl MessageType for PluginCharacteristicInfoResponse {
        fn message_type_id() -> MessageTypeId {
            MessageTypeId::PluginCharacteristicInfoResponse
        }
    }
}
