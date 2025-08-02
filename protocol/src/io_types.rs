//! Contains the basic types to reuse

use serde::{Deserialize, Serialize};

pub use host::*;
pub use plugin::*;

/// Host types
pub mod host {
    use crate::HostIO;
    use crate::IO;
    use crate::MAX_NAME_SIZE;
    use heapless::String;
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

    /// Host command. Configure service
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureService {
        /// Service UUID
        pub uuid: Uuid,
        /// Service name for identification
        pub name: String<MAX_NAME_SIZE>,
    }

    /// Host command. Configure characteristic
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandConfigureCharacteristic {}

    /// Host command. Get service info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandGetServiceInfo {
        /// Service UUID
        pub uuid: Uuid,
    }

    /// Host command. Get characteristic info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandGetCharacteristicInfo {}

    /// Host command. Get characteristic info
    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, HostIO)]
    pub struct HostCommandStartAdvertisement {
        /// Allow multiple central connections
        pub allow_multi_connect: bool,
    }
}

/// Plugin types
pub mod plugin {
    use crate::PluginIO;
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
}
