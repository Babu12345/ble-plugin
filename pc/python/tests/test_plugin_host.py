import uuid as uuid_module
from enum import Enum
from collections import namedtuple
from plugin_host.comms import parse_uuid_u16, serialize_command, deserialize_response
import plugin_host.protocol_pb2 as protocol_pb2

def test_type_serialization() -> None:
    # Test command serialization (things we send TO the device)
    cmd = protocol_pb2.HostCommandConfigurePeripheral(addr=bytes([0x12, 0x30, 0x00, 0x00, 0x00, 0x00]), name="Default peripheral")
    serialized = serialize_command(cmd)
    assert len(serialized) > 0, "Peripheral configuration serialization"

    cmd = protocol_pb2.HostCommandConfigureService(uuid=0x4560)
    serialized = serialize_command(cmd)
    assert len(serialized) > 0, "Host service configuration serialization"

    # Test that plugin data objects can be created (these come FROM the device, not serialized by us)
    data = protocol_pb2.PluginData(
        send_type=protocol_pb2.PluginDataSendType.NotifyType, 
        src_addr=bytes([0x12, 0x30, 0x00, 0x00, 0x00, 0x00]), 
        src_addr_type=protocol_pb2.BluetoothAddressType.Public, 
        characteristic_uuid=0x2A19, 
        service_uuid=0x180F, 
        data=bytes([0,1,2])
    )
    assert data.service_uuid == 0x180F, "Plugin data creation"

    # Test response objects can be created (these come FROM the device)
    response = protocol_pb2.PluginServiceInfoResponse(
        service_uuid=0x7890,
        exists=True,
        characteristic_uuids=[0x1111, 0x2222]
    )
    assert response.exists is True, "Service info response creation"

def test_new_commands_serialization() -> None:
    """Test serialization of the new command: ConfigureProfile"""
    
    # Test HostCommandConfigureProfile with Custom profile
    cmd = protocol_pb2.HostCommandConfigureProfile(profile=protocol_pb2.BleProfile.Custom)
    serialized = serialize_command(cmd)
    assert len(serialized) > 0, "Configure profile command serialization failed"
    
    # Test with different profile
    cmd = protocol_pb2.HostCommandConfigureProfile(profile=protocol_pb2.BleProfile.HeartRateMonitor)
    serialized = serialize_command(cmd)
    assert len(serialized) > 0, "Configure profile command serialization failed"

def test_new_commands_integration():
    """Test that new commands can be created and have proper message mappings"""
    from plugin_host.comms import serialize_command, USBHostDevice
    
    # Test ConfigureProfile command  
    cmd = protocol_pb2.HostCommandConfigureProfile(profile=protocol_pb2.BleProfile.Custom)
    serialized = serialize_command(cmd)
    assert len(serialized) > 5, "Should include message header"
    
    # Test that USBHostDevice has the new methods
    device = USBHostDevice()
    assert hasattr(device, 'configure_profile')
    assert callable(device.configure_profile)