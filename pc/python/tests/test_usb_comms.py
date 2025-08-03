import struct
from plugin_host.comms import (
    serialize_command,
    deserialize_response,
    DEFAULT_PACKET_SIZE
)
from plugin_host.types import (
    HostCommandConfigurePeripheral,
    HostCommandGetServiceInfo,
    PluginServiceInfoResponse,
    MESSAGE_MAGIC,
    MESSAGE_MAGIC_BYTES,
    MESSAGE_TYPE_ID_BYTES,
    DATA_BYTES_LENGTH_IN_BYTES,
    MESSAGE_HEADER_SIZE
)

def test_command_serialization_with_message_header() -> None:
    """Test serialization of protocol commands with full message header"""
    # Test HostCommandConfigurePeripheral
    cmd = HostCommandConfigurePeripheral(
        name="TestDevice",
        uuid="12345678-1234-1234-1234-123456789abc"
    )
    
    # Serialize command
    serialized = serialize_command(cmd)
    
    # Verify packet size is correct
    assert len(serialized) == DEFAULT_PACKET_SIZE, f"Expected packet size {DEFAULT_PACKET_SIZE}, got {len(serialized)}"
    
    # Verify magic number (first 2 bytes)
    magic = struct.unpack('<H', serialized[:MESSAGE_MAGIC_BYTES])[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:04X}, got 0x{magic:04X}"
    
    # Verify message type ID (byte 2)
    type_id = serialized[MESSAGE_MAGIC_BYTES]
    assert type_id == 0x01, f"Expected type ID 0x01 for HostCommandConfigurePeripheral, got 0x{type_id:02X}"
    
    # Verify data length (bytes 3-4)
    length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
    length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
    data_length = struct.unpack('<H', serialized[length_start:length_end])[0]
    assert data_length > 0, "Data length should be greater than 0"
    assert data_length < DEFAULT_PACKET_SIZE, f"Data length {data_length} should be less than packet size {DEFAULT_PACKET_SIZE}"

def test_service_command_serialization() -> None:
    """Test serialization of service info command"""
    service_cmd = HostCommandGetServiceInfo(
        uuid="87654321-4321-4321-4321-cba987654321"
    )
    
    serialized_service = serialize_command(service_cmd)
    
    # Verify packet size
    assert len(serialized_service) == DEFAULT_PACKET_SIZE, "Service command packet size incorrect"
    
    # Verify magic number
    magic = struct.unpack('<H', serialized_service[:MESSAGE_MAGIC_BYTES])[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:04X}, got 0x{magic:04X}"
    
    # Verify message type ID for HostCommandGetServiceInfo
    type_id = serialized_service[MESSAGE_MAGIC_BYTES]
    assert type_id == 0x05, f"Expected type ID 0x05 for HostCommandGetServiceInfo, got 0x{type_id:02X}"
    
    # Verify data length
    length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
    length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
    service_length = struct.unpack('<H', serialized_service[length_start:length_end])[0]
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

def test_message_header_protocol_roundtrip() -> None:
    """Test the message header protocol with round-trip serialization"""
    # Create a simple command
    cmd = HostCommandGetServiceInfo(uuid="test-uuid-123")
    
    # Serialize
    serialized = serialize_command(cmd)
    
    # Verify message header format
    assert len(serialized) == DEFAULT_PACKET_SIZE, "Serialized packet should match default size"
    
    # Verify magic number (first 2 bytes)
    magic = struct.unpack('<H', serialized[:MESSAGE_MAGIC_BYTES])[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:04X}, got 0x{magic:04X}"
    
    # Verify message type ID (byte 2)
    type_id = serialized[MESSAGE_MAGIC_BYTES]
    assert type_id == 0x05, f"Expected type ID 0x05 for HostCommandGetServiceInfo, got 0x{type_id:02X}"
    
    # Verify data length (bytes 3-4)
    length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
    length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
    data_length = struct.unpack('<H', serialized[length_start:length_end])[0]
    assert data_length > 0, "Data length should be positive"
    
    # Extract just the payload data for manual verification
    payload_start = MESSAGE_HEADER_SIZE
    payload_end = payload_start + data_length
    payload_data = serialized[payload_start:payload_end]
    assert len(payload_data) == data_length, f"Payload size {len(payload_data)} should match length field {data_length}"
    
    # Test deserialization using the same data
    deserialized = deserialize_response(serialized, HostCommandGetServiceInfo)
    
    assert deserialized.uuid == cmd.uuid, f"Round-trip failed: {deserialized.uuid} != {cmd.uuid}"
    assert deserialized == cmd, "Complete command objects should match after round-trip"