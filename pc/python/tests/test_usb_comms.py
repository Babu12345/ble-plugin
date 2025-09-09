import struct
from plugin_host.comms import (
    serialize_command,
    deserialize_response,
    DEFAULT_PACKET_SIZE,
    parse_uuid_u16,
    validate_mac_address
)
import plugin_host.protocol_pb2 as protocol_pb2
from plugin_host.constants import (
    MESSAGE_MAGIC,
    MESSAGE_MAGIC_BYTES,
    MESSAGE_TYPE_ID_BYTES,
    DATA_BYTES_LENGTH_IN_BYTES,
    MESSAGE_HEADER_SIZE
)

def test_command_serialization_with_message_header() -> None:
    """Test serialization of protocol commands with full message header"""
    # Test HostCommandConfigurePeripheral
    cmd = protocol_pb2.HostCommandConfigurePeripheral(
        name="TestDevice",
        addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc])  # MAC address as bytes
    )
    
    # Serialize command
    serialized = serialize_command(cmd)
    
    # Verify packet size is correct
    assert len(serialized) == DEFAULT_PACKET_SIZE, f"Expected packet size {DEFAULT_PACKET_SIZE}, got {len(serialized)}"
    
    # Verify magic number (first byte)
    magic = serialized[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:02X}, got 0x{magic:02X}"
    
    # Verify message type ID (bytes 1-2)
    type_id_bytes = serialized[MESSAGE_MAGIC_BYTES:MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES]
    type_id = struct.unpack('<H', type_id_bytes)[0]
    assert type_id == 0x01, f"Expected type ID 0x01 for HostCommandConfigurePeripheral, got 0x{type_id:02X}"
    
    # Verify data length (bytes 3-4)
    length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
    length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
    data_length = struct.unpack('<H', serialized[length_start:length_end])[0]
    assert data_length > 0, "Data length should be greater than 0"
    assert data_length < DEFAULT_PACKET_SIZE, f"Data length {data_length} should be less than packet size {DEFAULT_PACKET_SIZE}"

def test_service_command_serialization() -> None:
    """Test serialization of service info command"""
    service_cmd = protocol_pb2.HostCommandGetServiceInfo(
        uuid=0x8765
    )
    
    serialized_service = serialize_command(service_cmd)
    
    # Verify packet size
    assert len(serialized_service) == DEFAULT_PACKET_SIZE, "Service command packet size incorrect"
    
    # Verify magic number
    magic = serialized_service[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:02X}, got 0x{magic:02X}"
    
    # Verify message type ID for HostCommandGetServiceInfo
    type_id_bytes = serialized_service[MESSAGE_MAGIC_BYTES:MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES]
    type_id = struct.unpack('<H', type_id_bytes)[0]
    assert type_id == 0x05, f"Expected type ID 0x05 for HostCommandGetServiceInfo, got 0x{type_id:02X}"
    
    # Verify data length
    length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
    length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
    service_length = struct.unpack('<H', serialized_service[length_start:length_end])[0]
    assert service_length > 0, "Service command data length should be greater than 0"

def test_response_deserialization() -> None:
    """Test deserializing a mock response"""
    # Create a mock response
    mock_response = protocol_pb2.PluginServiceInfoResponse(
        service_uuid=0x8765,
        characteristic_uuids=[
            0x1111
        ],
        exists=True
    )
    
    # Serialize it (simulating what would come from device)
    serialized = serialize_command(mock_response)
    
    # Deserialize it back
    deserialized = deserialize_response(serialized, protocol_pb2.PluginServiceInfoResponse)
    
    assert deserialized.service_uuid == mock_response.service_uuid, "Service UUID mismatch"
    assert len(deserialized.characteristic_uuids) == len(mock_response.characteristic_uuids), "Characteristic count mismatch"
    assert deserialized.exists == mock_response.exists, "Exists flag mismatch"
    assert deserialized == mock_response, "Complete response objects should match"

def test_message_header_protocol_roundtrip() -> None:
    """Test the message header protocol with round-trip serialization"""
    # Create a simple command
    cmd = protocol_pb2.HostCommandGetServiceInfo(uuid=0x1234)
    
    # Serialize
    serialized = serialize_command(cmd)
    
    # Verify message header format
    assert len(serialized) == DEFAULT_PACKET_SIZE, "Serialized packet should match default size"
    
    # Verify magic number (first byte)
    magic = serialized[0]
    assert magic == MESSAGE_MAGIC, f"Expected magic 0x{MESSAGE_MAGIC:02X}, got 0x{magic:02X}"
    
    # Verify message type ID (bytes 1-2)
    type_id_bytes = serialized[MESSAGE_MAGIC_BYTES:MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES]
    type_id = struct.unpack('<H', type_id_bytes)[0]
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
    deserialized = deserialize_response(serialized, protocol_pb2.HostCommandGetServiceInfo)
    
    assert deserialized.uuid == cmd.uuid, f"Round-trip failed: {deserialized.uuid} != {cmd.uuid}"
    assert deserialized == cmd, "Complete command objects should match after round-trip"

def test_parse_uuid_u16_valid_values() -> None:
    """Test parse_uuid_u16 with valid u16 values"""
    # Test valid integers
    assert parse_uuid_u16(0) == 0
    assert parse_uuid_u16(1234) == 1234
    assert parse_uuid_u16(0xFFFF) == 65535  # Max u16
    
    # Test valid hex strings
    assert parse_uuid_u16('0x1234') == 0x1234
    assert parse_uuid_u16('0XABCD') == 0xABCD
    assert parse_uuid_u16('0xFFFF') == 65535
    
    # Test decimal strings
    assert parse_uuid_u16('1234') == 1234
    assert parse_uuid_u16('65535') == 65535
    
    # Test hex strings without prefix
    assert parse_uuid_u16('ABCD') == 0xABCD
    assert parse_uuid_u16('beef') == 0xBEEF

def test_parse_uuid_u16_invalid_values() -> None:
    """Test parse_uuid_u16 with invalid values that should raise ValueError"""
    import pytest
    
    # Test negative values
    with pytest.raises(ValueError, match="UUID value cannot be negative"):
        parse_uuid_u16(-1)
    
    with pytest.raises(ValueError, match="UUID value cannot be negative"):
        parse_uuid_u16('-1')
    
    # Test values exceeding u16 max
    with pytest.raises(ValueError, match="UUID value exceeds u16 maximum"):
        parse_uuid_u16(0x10000)  # One more than max u16
    
    with pytest.raises(ValueError, match="UUID value exceeds u16 maximum"):
        parse_uuid_u16('0x10000')
    
    with pytest.raises(ValueError, match="UUID value exceeds u16 maximum"):
        parse_uuid_u16('65536')  # One more than max u16 as decimal string
    
    # Test invalid type
    with pytest.raises(ValueError, match="Invalid UUID value type"):
        parse_uuid_u16([1, 2, 3])
    
    with pytest.raises(ValueError, match="Invalid UUID value type"):
        parse_uuid_u16(None)
    
    with pytest.raises(ValueError, match="Invalid UUID value type"):
        parse_uuid_u16({'uuid': 1234})

def test_validate_mac_address_valid() -> None:
    """Test validate_mac_address with valid MAC addresses"""
    # Valid public addresses
    assert validate_mac_address(bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])) is None
    assert validate_mac_address(bytes([0x00, 0x11, 0x22, 0x33, 0x44, 0x55])) is None
    
    # Valid Static Random addresses (MSB bits = 11)
    assert validate_mac_address(bytes([0xC0, 0x01, 0x02, 0x03, 0x04, 0x05])) is None
    assert validate_mac_address(bytes([0xFF, 0x11, 0x22, 0x33, 0x44, 0x55])) is None
    
    # Valid Non-Resolvable Private addresses (MSB bits = 00)
    assert validate_mac_address(bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05])) is None
    assert validate_mac_address(bytes([0x3F, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE])) is None
    
    # Test with different input types
    assert validate_mac_address(bytearray([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])) is None
    assert validate_mac_address([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]) is None

def test_validate_mac_address_invalid_length() -> None:
    """Test validate_mac_address with invalid lengths"""
    # Too short
    result = validate_mac_address(bytes([0x12, 0x34, 0x56, 0x78, 0x9A]))
    assert result is not None
    assert "MAC address must be exactly 6 bytes" in result
    assert "Got 5 bytes" in result
    
    # Too long
    result = validate_mac_address(bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE]))
    assert result is not None
    assert "MAC address must be exactly 6 bytes" in result
    assert "Got 7 bytes" in result
    
    # Empty
    result = validate_mac_address(bytes([]))
    assert result is not None
    assert "Got 0 bytes" in result

def test_validate_mac_address_invalid_patterns() -> None:
    """Test validate_mac_address with invalid random address patterns"""
    # Invalid MSB pattern 10
    result = validate_mac_address(bytes([0x80, 0x11, 0x22, 0x33, 0x44, 0x55]))
    assert result is not None
    assert "MSB bits are 10 (binary)" in result
    
    # Invalid Resolvable Private (MSB pattern 01)
    result = validate_mac_address(bytes([0x40, 0x11, 0x22, 0x33, 0x44, 0x55]))
    assert result is not None
    assert "Resolvable Private addresses not allowed" in result
    
    # Invalid Static Random with all zeros in remaining bits
    result = validate_mac_address(bytes([0xC0, 0x00, 0x00, 0x00, 0x00, 0x00]))
    assert result is not None
    assert "Invalid Static Random address" in result
    assert "remaining 46 bits must contain at least one 0 and at least one 1" in result
    
    # Invalid Static Random with all ones in remaining bits
    result = validate_mac_address(bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]))
    assert result is not None
    assert "Invalid Static Random address" in result
    assert "remaining 46 bits must contain at least one 0 and at least one 1" in result
    
    # Invalid Non-Resolvable Private with all zeros
    result = validate_mac_address(bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]))
    assert result is not None
    assert "Invalid Non-Resolvable Private address" in result
    assert "remaining 46 bits must contain at least one 0 and at least one 1" in result

    # Invalid Non-Resolvable Private with all ones in remaining bits
    result = validate_mac_address(bytes([0x3F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]))
    assert result is not None
    assert "Invalid Non-Resolvable Private address" in result
    assert "remaining 46 bits must contain at least one 0 and at least one 1" in result