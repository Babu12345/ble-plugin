import struct
from plugin_host.comms import (
    serialize_command,
    deserialize_response,
    DATA_BYTES_LENGTH_IN_BYTES,
    DEFAULT_PACKET_SIZE
)
from plugin_host.types import (
    HostCommandConfigurePeripheral,
    HostCommandGetServiceInfo,
    PluginServiceInfoResponse
)

def test_command_serialization_with_length_prefix() -> None:
    """Test serialization of protocol commands with length prefix"""
    # Test HostCommandConfigurePeripheral
    cmd = HostCommandConfigurePeripheral(
        name="TestDevice",
        uuid="12345678-1234-1234-1234-123456789abc"
    )
    
    # Serialize command
    serialized = serialize_command(cmd)
    
    # Verify packet size is correct
    assert len(serialized) == DEFAULT_PACKET_SIZE, f"Expected packet size {DEFAULT_PACKET_SIZE}, got {len(serialized)}"
    
    # Verify length prefix is present and correct
    data_length = struct.unpack('<H', serialized[:DATA_BYTES_LENGTH_IN_BYTES])[0]
    assert data_length > 0, "Data length should be greater than 0"
    assert data_length < DEFAULT_PACKET_SIZE, f"Data length {data_length} should be less than packet size {DEFAULT_PACKET_SIZE}"

def test_service_command_serialization() -> None:
    """Test serialization of service info command"""
    service_cmd = HostCommandGetServiceInfo(
        uuid="87654321-4321-4321-4321-cba987654321"
    )
    
    serialized_service = serialize_command(service_cmd)
    service_length = struct.unpack('<H', serialized_service[:DATA_BYTES_LENGTH_IN_BYTES])[0]
    
    assert len(serialized_service) == DEFAULT_PACKET_SIZE, "Service command packet size incorrect"
    assert service_length > 0, "Service command data length should be greater than 0"

def test_response_deserialization() -> None:
    """Test deserializing a mock response"""
    # Create a mock response
    mock_response = PluginServiceInfoResponse(
        service_uuid="87654321-4321-4321-4321-cba987654321",
        characteristic_uuids=["char1-uuid", "char2-uuid"],
        exists=True
    )
    
    # Serialize it (simulating what would come from device)
    serialized = serialize_command(mock_response)
    
    # Deserialize it back
    deserialized = deserialize_response(serialized, PluginServiceInfoResponse)
    
    assert deserialized.service_uuid == mock_response.service_uuid, "Service UUID mismatch"
    assert len(deserialized.characteristic_uuids) == len(mock_response.characteristic_uuids), "Characteristic count mismatch"
    assert deserialized.exists == mock_response.exists, "Exists flag mismatch"
    assert deserialized == mock_response, "Complete response objects should match"

def test_length_prefix_protocol_roundtrip() -> None:
    """Test the length prefix protocol with round-trip serialization"""
    # Create a simple command
    cmd = HostCommandGetServiceInfo(uuid="test-uuid-123")
    
    # Serialize
    serialized = serialize_command(cmd)
    
    # Manually inspect the length prefix
    length_prefix = serialized[:DATA_BYTES_LENGTH_IN_BYTES]
    data_length = struct.unpack('<H', length_prefix)[0]
    
    # Verify length prefix format
    assert len(length_prefix) == DATA_BYTES_LENGTH_IN_BYTES, "Length prefix should be 2 bytes"
    assert data_length > 0, "Data length should be positive"
    
    # Extract just the payload data for manual verification
    payload_data = serialized[DATA_BYTES_LENGTH_IN_BYTES:DATA_BYTES_LENGTH_IN_BYTES + data_length]
    assert len(payload_data) == data_length, f"Payload size {len(payload_data)} should match length prefix {data_length}"
    
    # Test deserialization using the same data
    deserialized = deserialize_response(serialized, HostCommandGetServiceInfo)
    
    assert deserialized.uuid == cmd.uuid, f"Round-trip failed: {deserialized.uuid} != {cmd.uuid}"
    assert deserialized == cmd, "Complete command objects should match after round-trip"