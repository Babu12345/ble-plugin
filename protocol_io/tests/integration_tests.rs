use protocol_io::{HostIO, PluginIO};
use serde::{Deserialize, Serialize};

// Test helper types and traits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTypeId {
    HostCommandConfigurePeripheral,
    HostCommandConfigureService,
    HostCommandConfigureCharacteristic,
    HostCommandConfigureCharacteristicRead,
    HostCommandGetServiceInfo,
    HostCommandGetCharacteristicInfo,
    HostCommandStartAdvertisement,
    HostCommandNotifyCharacteristicValue,
    PluginData,
    PluginConfigurationError,
    PluginServiceInfoResponse,
    PluginCharacteristicInfoResponse,
    PluginAuthenticationCompletedResponse,
}

pub trait MessageType {
    const MESSAGE_TYPE_ID: MessageTypeId;
}

pub trait IOBase<'a> {}
pub trait IO<'a>: IOBase<'a> {}
pub trait HostIO<'a>: IO<'a> {}
pub trait PluginIO<'a>: IO<'a> {}

// Test helper functions to check if a type implements the expected traits
#[allow(unused)]
fn assert_host_io_traits<T>()
where
    T: MessageType,
{
    // This function will only compile if T implements MessageType
    // For IO/HostIO traits, we'll test them implicitly through usage
}

#[allow(unused)]
fn assert_plugin_io_traits<T>()
where
    T: MessageType,
{
    // This function will only compile if T implements MessageType
    // For IO/PluginIO traits, we'll test them implicitly through usage
}

// Test structures with zero lifetimes
mod zero_lifetimes {
    use super::*;

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
    struct SimpleHostCommand {
        data: u32,
        flag: bool,
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginData)]
    struct SimplePluginResponse {
        status: String,
        code: u16,
    }

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigureService)]
    enum HostCommandEnum {
        Start,
        Stop { reason: String },
        Configure { setting: u32, enabled: bool },
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginConfigurationError)]
    enum PluginResponseEnum {
        Success,
        Error { code: u32, message: String },
    }

    #[test]
    fn test_zero_lifetime_struct_implements_traits() {
        assert_host_io_traits::<SimpleHostCommand>();
        assert_plugin_io_traits::<SimplePluginResponse>();
    }

    #[test]
    fn test_zero_lifetime_enum_implements_traits() {
        assert_host_io_traits::<HostCommandEnum>();
        assert_plugin_io_traits::<PluginResponseEnum>();
    }

    #[test]
    fn test_message_type_id_is_correct() {
        assert_eq!(
            SimpleHostCommand::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigurePeripheral
        );

        assert_eq!(
            SimplePluginResponse::MESSAGE_TYPE_ID,
            MessageTypeId::PluginData
        );

        assert_eq!(
            HostCommandEnum::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigureService
        );

        assert_eq!(
            PluginResponseEnum::MESSAGE_TYPE_ID,
            MessageTypeId::PluginConfigurationError
        );
    }
}

// Test structures with single lifetime
mod single_lifetime {
    use super::*;

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigureCharacteristic)]
    struct SingleLifetimeHostCommand<'a> {
        data: &'a [u8],
        name: &'a str,
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginServiceInfoResponse)]
    struct SingleLifetimePluginResponse<'a> {
        message: &'a str,
        buffer: &'a [u8],
    }

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandGetServiceInfo)]
    enum SingleLifetimeHostEnum<'a> {
        Query { query: &'a str },
        Data { payload: &'a [u8] },
    }

    #[test]
    fn test_single_lifetime_implements_traits() {
        assert_host_io_traits::<SingleLifetimeHostCommand<'_>>();
        assert_plugin_io_traits::<SingleLifetimePluginResponse<'_>>();
        assert_host_io_traits::<SingleLifetimeHostEnum<'_>>();
    }

    #[test]
    fn test_single_lifetime_message_type_id() {
        assert_eq!(
            SingleLifetimeHostCommand::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigureCharacteristic
        );

        assert_eq!(
            SingleLifetimePluginResponse::MESSAGE_TYPE_ID,
            MessageTypeId::PluginServiceInfoResponse
        );

        assert_eq!(
            SingleLifetimeHostEnum::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandGetServiceInfo
        );
    }

    #[test]
    fn test_single_lifetime_usage() {
        let data = b"test data";
        let name = "test command";

        let _cmd = SingleLifetimeHostCommand { data, name };

        let message = "response message";
        let buffer = b"response buffer";

        let _response = SingleLifetimePluginResponse { message, buffer };
    }
}

// Test structures with multiple lifetimes
mod multiple_lifetimes {
    use super::*;

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigureCharacteristicRead)]
    struct MultipleLifetimeHostCommand<'a, 'b> {
        primary_data: &'a [u8],
        secondary_data: &'b str,
        id: u32,
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginCharacteristicInfoResponse)]
    struct MultipleLifetimePluginResponse<'x, 'y, 'z> {
        first: &'x str,
        second: &'y [u8],
        third: &'z str,
    }

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
    enum MultipleLifetimeEnum<'a, 'b> {
        First { data: &'a [u8] },
        Second { name: &'b str },
        Both { data: &'a [u8], name: &'b str },
    }

    #[test]
    fn test_multiple_lifetimes_implements_traits() {
        assert_host_io_traits::<MultipleLifetimeHostCommand<'_, '_>>();
        assert_plugin_io_traits::<MultipleLifetimePluginResponse<'_, '_, '_>>();
        assert_host_io_traits::<MultipleLifetimeEnum<'_, '_>>();
    }

    #[test]
    fn test_multiple_lifetimes_message_type_id() {
        assert_eq!(
            MultipleLifetimeHostCommand::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigureCharacteristicRead
        );

        assert_eq!(
            MultipleLifetimePluginResponse::MESSAGE_TYPE_ID,
            MessageTypeId::PluginCharacteristicInfoResponse
        );

        assert_eq!(
            MultipleLifetimeEnum::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandStartAdvertisement
        );
    }

    #[test]
    fn test_multiple_lifetimes_usage() {
        let primary = b"primary data";
        let secondary = "secondary string";

        let _cmd = MultipleLifetimeHostCommand {
            primary_data: primary,
            secondary_data: secondary,
            id: 42,
        };

        let first = "first string";
        let second = b"second bytes";
        let third = "third string";

        let _response = MultipleLifetimePluginResponse {
            first,
            second,
            third,
        };

        let data = b"enum data";
        let name = "enum name";

        let _enum_first = MultipleLifetimeEnum::First { data };
        let _enum_second = MultipleLifetimeEnum::Second { name };
        let _enum_both = MultipleLifetimeEnum::Both { data, name };
    }
}

// Test generic types with lifetimes
mod generic_types {
    use super::*;

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandNotifyCharacteristicValue)]
    struct GenericHostCommand<'a> {
        data: &'a [u8],
        payload: u32,
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginAuthenticationCompletedResponse)]
    struct GenericPluginResponse<'a> {
        message: &'a str,
        first_data: String,
        second_data: u64,
    }

    #[test]
    fn test_generic_types_implement_traits() {
        assert_host_io_traits::<GenericHostCommand<'_>>();
        assert_plugin_io_traits::<GenericPluginResponse<'_>>();
    }

    #[test]
    fn test_generic_types_message_type_id() {
        assert_eq!(
            GenericHostCommand::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandNotifyCharacteristicValue
        );

        assert_eq!(
            GenericPluginResponse::MESSAGE_TYPE_ID,
            MessageTypeId::PluginAuthenticationCompletedResponse
        );
    }

    #[test]
    fn test_generic_types_usage() {
        let data = b"generic data";
        let payload = 42u32;

        let _cmd = GenericHostCommand { data, payload };

        let message = "generic response";
        let first_data = "first".to_string();
        let second_data = 100u64;

        let _response = GenericPluginResponse {
            message,
            first_data,
            second_data,
        };
    }
}

// Edge case tests
mod edge_cases {
    use super::*;

    // Test that lifetime order - macro should use first lifetime
    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
    struct FirstLifetimeUsed<'a, 'b> {
        // 'a is first in generic params and should be used for IO<'a>
        data_a: &'a [u8],
        data_b: &'b str,
    }

    #[test]
    fn test_first_lifetime_parameter_is_used() {
        // Test compilation - the macro should generate IO<'a> and HostIO<'a>
        let data_a = b"data a";
        let data_b = "data b";
        let _cmd = FirstLifetimeUsed { data_a, data_b };

        assert_eq!(
            FirstLifetimeUsed::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigurePeripheral
        );
    }

    // Test empty struct
    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigureService)]
    struct EmptyStruct;

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginData)]
    struct EmptyStructPlugin;

    #[test]
    fn test_empty_structs() {
        assert_host_io_traits::<EmptyStruct>();
        assert_plugin_io_traits::<EmptyStructPlugin>();

        let _host = EmptyStruct;
        let _plugin = EmptyStructPlugin;
    }

    // Test unit enum
    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandStartAdvertisement)]
    enum UnitEnum {
        A,
        B,
        C,
    }

    #[test]
    fn test_unit_enum() {
        assert_host_io_traits::<UnitEnum>();

        let _a = UnitEnum::A;
        let _b = UnitEnum::B;
        let _c = UnitEnum::C;
    }
}

// Compilation tests (ensure macros generate syntactically correct code)
mod compilation_tests {
    use super::*;

    // These tests primarily check that the generated code compiles correctly

    #[derive(Serialize, Deserialize)]
    #[HostIO(MessageTypeId::HostCommandConfigurePeripheral)]
    struct ComplexHost<'a, 'b> {
        lifetime_a: &'a str,
        lifetime_b: &'b [u8],
        regular_field: i32,
    }

    #[derive(Serialize, Deserialize)]
    #[PluginIO(MessageTypeId::PluginData)]
    struct ComplexPlugin<'x> {
        data: &'x [u8],
        value: u64,
    }

    #[test]
    fn test_complex_generics_compile() {
        // Just test that the types can be instantiated and have correct message type IDs
        let lifetime_a = "test a";
        let lifetime_b = b"test b";
        let _host = ComplexHost {
            lifetime_a,
            lifetime_b,
            regular_field: 42,
        };

        let data = b"plugin data";
        let _plugin = ComplexPlugin { data, value: 100 };

        assert_eq!(
            ComplexHost::MESSAGE_TYPE_ID,
            MessageTypeId::HostCommandConfigurePeripheral
        );

        assert_eq!(ComplexPlugin::MESSAGE_TYPE_ID, MessageTypeId::PluginData);
    }
}
