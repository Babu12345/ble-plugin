import attrs2bin
import uuid as uuid_module
from enum import Enum
from collections import namedtuple
from plugin_host.comms import parse_uuid_u16
from plugin_host.generated_types import *

def test_type_serialization() -> None:
    cmd = HostCommandConfigurePeripheral(addr=[0x12, 0x30, 0x00, 0x00, 0x00, 0x00], name="Default peripheral")
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigurePeripheral)
    assert cmd == deserialized, "Peripheral configuration serialization"

    cmd = HostCommandConfigureService(uuid=0x4560)
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigureService)
    assert cmd == deserialized, "Host service configuration deserialization"

    enum = PluginDataSendType.Notify
    serialized = attrs2bin.serialize(enum)
    deserialized = attrs2bin.deserialize(serialized, PluginDataSendType)    
    assert enum is deserialized, "Host data send type enum deserialization"

    data = PluginData(send_type=PluginDataSendType.Notify, src_addr=[0x12, 0x30, 0x00, 0x00, 0x00, 0x00], src_addr_type=BluetoothAddressType.Public, data=bytes([0,1,2]))
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginData)
    assert data == deserialized, "Host data transmission"

    data = PluginServiceInfoResponse(
        service_uuid=0x7890,
        exists=True,
        characteristic_uuids= [
            0x1111,
            0x2222
        ],
    )
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginServiceInfoResponse)
    assert data == deserialized, "Host data transmission"

def test_new_commands_serialization() -> None:
    """Test serialization of the new commands: ClearAllServices and ConfigureProfile"""
    
    # Test HostCommandClearAllServices (empty struct)
    cmd = HostCommandClearAllServices()
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandClearAllServices)
    assert cmd == deserialized, "Clear all services command serialization failed"
    
    # Test HostCommandConfigureProfile with Custom profile
    cmd = HostCommandConfigureProfile(profile=BLEProfile.Custom)
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigureProfile)
    assert cmd == deserialized, "Configure profile command serialization failed"
    
    # Test BLEProfile enum
    profile = BLEProfile.Custom
    serialized = attrs2bin.serialize(profile)
    deserialized = attrs2bin.deserialize(serialized, BLEProfile)
    assert profile is deserialized, "BLE profile enum serialization failed"

def test_new_commands_integration():
    """Test that new commands can be created and have proper message mappings"""
    from plugin_host.comms import serialize_command, MESSAGE_TYPE_MAP, USBHostDevice
    
    # Test ClearAllServices command
    cmd = HostCommandClearAllServices()
    serialized = serialize_command(cmd)
    assert len(serialized) > 5, "Should include message header"
    
    # Test ConfigureProfile command  
    cmd = HostCommandConfigureProfile(profile=BLEProfile.Custom)
    serialized = serialize_command(cmd)
    assert len(serialized) > 5, "Should include message header"
    
    # Test that commands are in MESSAGE_TYPE_MAP
    assert HostCommandClearAllServices in MESSAGE_TYPE_MAP
    assert HostCommandConfigureProfile in MESSAGE_TYPE_MAP
    
    # Test that USBHostDevice has the new methods
    device = USBHostDevice()
    assert hasattr(device, 'clear_all_services')
    assert hasattr(device, 'configure_profile')
    assert callable(device.clear_all_services)
    assert callable(device.configure_profile)