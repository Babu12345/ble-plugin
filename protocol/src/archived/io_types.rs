//! Contains the basic types to reuse
//! Warning. If you use a fixed size array then there will be no length prefix
//! when serializing. Use heapless::Vec instead to ensure the length is prefixed
//! and deserialized correctly from different languages and you want to ensure a max size
//! is enforced.
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
    use protocol_io::HostIO;
    use serde::{Deserialize, Serialize};

    /// Host command. Configure peripheral
    // Random Bluetooth addresses must follow specific MSB bit patterns:
    // - Static Random: MSB bits = 11 (0xC0-0xFF), remaining 46 bits must have at least one 0 and one 1
    // - Non-Resolvable Private: MSB bits = 00 (0x00-0x3F), remaining 46 bits must have at least one 0 and one 1
    // Invalid pattern like 0xa1a1a1a1a1a1 has MSB bits = 10 (not allowed)
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
    pub struct HostCommandConfigurePeripheral {
        /// Peripheral name
        pub name: String<MAX_NAME_SIZE>,
        /// Peripheral addr
        pub addr: heapless::Vec<u8, 6>,
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
        /// Service UUID (u16)
        pub uuid: u16,
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
        /// Characteristic UUID (u16)
        pub uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
        /// Properties of the characteristic (read, write, notify, etc.)
        pub properties: heapless::Vec<BLEProperties, MAX_PROPERTIES>, // Assuming max 4 properties per characteristic
    }

    /// Host command. Configure characteristic read
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigureCharacteristicRead)]
    pub struct HostCommandConfigureCharacteristicRead {
        /// Characteristic UUID (u16)
        pub uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
        /// Read value
        pub value: heapless::Vec<u8, MAX_NAME_SIZE>,
    }

    /// Host command. Get service info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandGetServiceInfo)]
    pub struct HostCommandGetServiceInfo {
        /// Service UUID (u16)
        pub uuid: u16,
    }

    /// Host command. Get characteristic info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandGetCharacteristicInfo)]
    pub struct HostCommandGetCharacteristicInfo {
        /// Characteristic UUID (u16)
        pub characteristic_uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
    }

    /// Host command. Start advertisement. TODO: Allow selecting which services you want to advertise
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
    pub struct HostCommandStartAdvertisement {
        /// Allow multiple central connections
        pub allow_multi_connect: bool,
    }

    /// Host command. Stop all advertisement
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandStopAdvertisement)]
    pub struct HostCommandStopAdvertisement {}

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
        pub address: heapless::Vec<u8, 6>,
        /// Address type
        pub address_type: BluetoothAddressType,
        /// Characteristic UUID (u16)
        pub characteristic_uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
        /// Value to notify
        pub value: heapless::Vec<u8, MAX_NAME_SIZE>,
    }

    /// Profile type enumeration for preconfigured BLE profiles
    #[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    pub enum BLEProfile {
        /// Custom profile - user-defined services
        Custom = 0,
        /// Heart Rate Monitor profile
        HeartRateMonitor = 1,
        /// Battery Service profile
        BatteryService = 2,
        /// Device Information Service profile
        DeviceInformation = 3,
    }

    /// Host command. Configure BLE profile
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[HostIO(MessageTypeId::HostCommandConfigureProfile)]
    pub struct HostCommandConfigureProfile {
        /// Profile type to configure
        pub profile: BLEProfile,
    }
}

/// Plugin types
pub mod plugin {
    use crate::PluginIO;
    use crate::{MessageType, MessageTypeId};
    use protocol_io::PluginIO;

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
        /// Source peripheral addr that this data is orginating from.
        pub src_addr: heapless::Vec<u8, 6>,
        /// Address type of the source peripheral
        pub src_addr_type: BluetoothAddressType,
        /// Send type of the data
        pub send_type: PluginDataSendType,
        /// Characteristic UUID (u16)
        pub characteristic_uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
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
        /// The peripheral address is invalid
        InvalidPeripheralUuid = 1,
        /// The service UUID (u16) is invalid
        InvalidServiceUuid = 2,
        /// The characteristic UUID (u16) is invalid
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
        /// Service UUID (u16)
        pub service_uuid: u16,
        /// List of characteristic UUIDs (u16) in this service
        pub characteristic_uuids: heapless::Vec<u16, MAX_CHARACTERISTICS_PER_SERVICE>, // Assuming max 16 characteristics per service
        /// Whether the service exists
        pub exists: bool,
    }

    /// Characteristic information response
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
    #[PluginIO(MessageTypeId::PluginCharacteristicInfoResponse)]
    pub struct PluginCharacteristicInfoResponse {
        /// Characteristic UUID (u16)
        pub characteristic_uuid: u16,
        /// Service UUID (u16) this characteristic belongs to
        pub service_uuid: u16,
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
        pub address: heapless::Vec<u8, 6>,
        /// Address type
        pub address_type: BluetoothAddressType,
        /// Whether the authentication was successful
        pub success: bool,
    }
}

/// Message type identifiers for efficient command dispatch
///
/// This enum defines unique identifiers for each message type in the protocol,
/// enabling O(1) message dispatch without trial-and-error deserialization.
/// The type IDs are organized into logical ranges for easy identification.
///
/// ## Type ID Ranges
///
/// - **0x01-0x7F**: Host commands sent to plugin device
/// - **0x80-0xFF**: Plugin responses sent to host device
///
/// ## Usage
///
/// ```rust
/// use protocol::MessageTypeId;
///
/// // Check if message is a host command
/// let is_host_command = (type_id as u8) < 0x80;
///
/// // Check if message is a plugin response  
/// let is_plugin_response = (type_id as u8) >= 0x80;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
#[repr(u8)]
pub enum MessageTypeId {
    // Host Commands (0x01-0x0F)
    /// Configure BLE peripheral device with name and 6-byte address
    HostCommandConfigurePeripheral = 0x01,

    /// Create a new BLE service with specified u16 UUID
    HostCommandConfigureService = 0x02,

    /// Create a BLE characteristic with properties
    HostCommandConfigureCharacteristic = 0x03,

    /// Configure characteristic for read operations with default value
    HostCommandConfigureCharacteristicRead = 0x04,

    /// Query information about a BLE service
    HostCommandGetServiceInfo = 0x05,

    /// Query information about a BLE characteristic
    HostCommandGetCharacteristicInfo = 0x06,

    /// Start BLE advertising with optional multi-connect support
    HostCommandStartAdvertisement = 0x07,

    /// Send notification/indication to connected BLE client
    HostCommandNotifyCharacteristicValue = 0x08,

    /// Configure BLE security settings (pairing, bonding, passkey, etc.)
    HostCommandConfigurePeripheralSecurity = 0x09,

    /// Configure BLE profile with predefined services and characteristics
    HostCommandConfigureProfile = 0x0A,

    /// Stop BLE advertising
    HostCommandStopAdvertisement = 0x0B,

    // Plugin Responses (0x80+)
    /// Data forwarded from BLE client to BLE plugin
    PluginData = 0x80,

    /// Configuration error response from plugin
    PluginConfigurationError = 0x81,

    /// Service information response with characteristic list
    PluginServiceInfoResponse = 0x82,

    /// Characteristic information response with properties
    PluginCharacteristicInfoResponse = 0x83,

    /// Authentication completed response from plugin
    PluginAuthenticationCompletedResponse = 0x84,
}

/// Error type for converting from u8 to MessageTypeId
pub struct InvalidMessageTypeIdForConversion;

impl TryFrom<u8> for MessageTypeId {
    type Error = InvalidMessageTypeIdForConversion;
    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        MessageTypeId::iter()
            .find(|&x| x as u8 == value)
            .ok_or(InvalidMessageTypeIdForConversion)
    }
}
