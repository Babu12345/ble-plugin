# Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
#
# This source code is proprietary and confidential. Unauthorized copying,
# modification, distribution, or use of this software is strictly prohibited.

"""
Test protobuf encoding/decoding functionality in comms.py

This test verifies that the protobuf serialization and deserialization functions
work correctly with proper message headers and data integrity.
"""
import pytest
import struct
from unittest.mock import Mock, patch
from plugin_host.comms import (
    serialize_command,
    deserialize_response,
    USBCommunicationError,
    usb_send_command,
    usb_receive_response
)
from plugin_host.constants import (
    MESSAGE_MAGIC,
    MESSAGE_HEADER_SIZE,
    DEFAULT_PACKET_SIZE
)
import plugin_host.protocol_pb2 as protocol_pb2


class TestProtobufSerialization:
    """Test protobuf serialization functionality"""
    
    def test_serialize_host_command_configure_peripheral(self):
        """Test serializing HostCommandConfigurePeripheral with protobuf"""
        # Create a protobuf command
        cmd = protocol_pb2.HostCommandConfigurePeripheral()
        cmd.name = "TestDevice"
        cmd.addr = b'\x12\x34\x56\x78\x9a\xbc'  # 6-byte MAC address
        
        # Serialize using protobuf
        result = serialize_command(cmd)
        
        # Verify the result is exactly the packet size
        assert len(result) == DEFAULT_PACKET_SIZE
        
        # Verify magic number (first byte)
        magic = result[0]
        assert magic == MESSAGE_MAGIC
        
        # Verify message type ID (2nd and 3rd bytes, little-endian)
        type_id = struct.unpack('<H', result[1:3])[0]
        expected_type_id = protocol_pb2.MessageTypeId.TypeHostCommandConfigurePeripheral
        assert type_id == expected_type_id
        
        # Verify length (4th and 5th bytes, little-endian)
        length = struct.unpack('<H', result[3:5])[0]
        
        # Extract and verify the protobuf data
        protobuf_data = result[MESSAGE_HEADER_SIZE:MESSAGE_HEADER_SIZE + length]
        
        # Deserialize the protobuf data to verify it's correct
        deserialized_cmd = protocol_pb2.HostCommandConfigurePeripheral()
        deserialized_cmd.ParseFromString(protobuf_data)
        
        assert deserialized_cmd.name == "TestDevice"
        assert deserialized_cmd.addr == b'\x12\x34\x56\x78\x9a\xbc'
        
        # Verify the rest is padding (zeros)
        padding = result[MESSAGE_HEADER_SIZE + length:]
        assert all(b == 0 for b in padding)
    
    def test_serialize_host_command_get_service_info(self):
        """Test serializing HostCommandGetServiceInfo with protobuf"""
        cmd = protocol_pb2.HostCommandGetServiceInfo()
        cmd.uuid = 0x1234
        
        result = serialize_command(cmd)
        
        assert len(result) == DEFAULT_PACKET_SIZE
        
        # Verify magic and type ID
        magic = result[0]
        assert magic == MESSAGE_MAGIC
        expected_type_id = protocol_pb2.MessageTypeId.TypeHostCommandGetServiceInfo
        type_id = struct.unpack('<H', result[1:3])[0]
        assert type_id == expected_type_id
        
        # Verify the data can be deserialized correctly
        length = struct.unpack('<H', result[3:5])[0]
        protobuf_data = result[MESSAGE_HEADER_SIZE:MESSAGE_HEADER_SIZE + length]
        
        deserialized_cmd = protocol_pb2.HostCommandGetServiceInfo()
        deserialized_cmd.ParseFromString(protobuf_data)
        assert deserialized_cmd.uuid == 0x1234
    
    def test_serialize_host_command_configure_characteristic(self):
        """Test serializing HostCommandConfigureCharacteristic with protobuf"""
        cmd = protocol_pb2.HostCommandConfigureCharacteristic()
        cmd.uuid = 0x2345
        cmd.service_uuid = 0x6789
        cmd.properties.extend([1, 2, 4])  # READ, WRITE, NOTIFY
        
        result = serialize_command(cmd)
        
        assert len(result) == DEFAULT_PACKET_SIZE
        
        # Verify the data
        length = struct.unpack('<H', result[3:5])[0]
        protobuf_data = result[MESSAGE_HEADER_SIZE:MESSAGE_HEADER_SIZE + length]
        
        deserialized_cmd = protocol_pb2.HostCommandConfigureCharacteristic()
        deserialized_cmd.ParseFromString(protobuf_data)
        assert deserialized_cmd.uuid == 0x2345
        assert deserialized_cmd.service_uuid == 0x6789
        assert list(deserialized_cmd.properties) == [1, 2, 4]
    
    def test_serialize_unknown_command_type(self):
        """Test that unknown command types raise appropriate error"""
        class UnknownCommand:
            pass
        
        unknown_cmd = UnknownCommand()
        
        with pytest.raises(USBCommunicationError, match="Unknown protobuf message type"):
            serialize_command(unknown_cmd)
    
    def test_serialize_command_too_large(self):
        """Test that overly large commands are rejected"""
        cmd = protocol_pb2.HostCommandConfigurePeripheral()
        cmd.name = "x" * (DEFAULT_PACKET_SIZE - MESSAGE_HEADER_SIZE + 1)  # Too large
        
        with pytest.raises(USBCommunicationError, match="exceeds packet size"):
            serialize_command(cmd)


class TestProtobufDeserialization:
    """Test protobuf deserialization functionality"""
    
    def test_deserialize_plugin_data(self):
        """Test deserializing PluginData response with protobuf"""
        # Create a protobuf response
        response = protocol_pb2.PluginData()
        response.src_addr = b'\x12\x34\x56\x78\x9a\xbc'
        response.src_addr_type = 1  # PUBLIC
        response.send_type = 3  # WRITE_TYPE
        response.characteristic_uuid = 0x1234
        response.service_uuid = 0x5678
        response.data = b"test_data"
        
        # Serialize it to protobuf format
        protobuf_data = response.SerializeToString()
        
        # Create complete message with header
        header = bytearray()
        header.append(MESSAGE_MAGIC)
        header.extend(struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginData))
        header.extend(struct.pack('<H', len(protobuf_data)))
        
        complete_message = bytes(header) + protobuf_data
        # Pad to packet size
        padded_message = complete_message + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_message))
        
        # Deserialize using protobuf
        result = deserialize_response(padded_message)
        
        assert isinstance(result, protocol_pb2.PluginData)
        assert result.src_addr == b'\x12\x34\x56\x78\x9a\xbc'
        assert result.src_addr_type == 1
        assert result.send_type == 3
        assert result.characteristic_uuid == 0x1234
        assert result.service_uuid == 0x5678
        assert result.data == b"test_data"
    
    def test_deserialize_service_info_response(self):
        """Test deserializing PluginServiceInfoResponse with protobuf"""
        response = protocol_pb2.PluginServiceInfoResponse()
        response.service_uuid = 0x1234
        response.characteristic_uuids.extend([0x5678, 0x9abc])
        response.exists = True
        
        protobuf_data = response.SerializeToString()
        
        # Create complete message with header
        header = bytearray()
        header.append(MESSAGE_MAGIC)
        header.extend(struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginServiceInfoResponse))
        header.extend(struct.pack('<H', len(protobuf_data)))
        
        complete_message = bytes(header) + protobuf_data
        padded_message = complete_message + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_message))
        
        result = deserialize_response(padded_message)
        
        assert isinstance(result, protocol_pb2.PluginServiceInfoResponse)
        assert result.service_uuid == 0x1234
        assert list(result.characteristic_uuids) == [0x5678, 0x9abc]
        assert result.exists is True
    
    def test_deserialize_configuration_error(self):
        """Test deserializing PluginConfigurationError with protobuf"""
        response = protocol_pb2.PluginConfigurationError()
        response.error_type = protocol_pb2.PluginConfigurationErrorType.AdvertisementWithoutPeripheralConfiguration
        
        protobuf_data = response.SerializeToString()
        
        header = bytearray()
        header.append(MESSAGE_MAGIC)
        header.extend(struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginConfigurationError))
        header.extend(struct.pack('<H', len(protobuf_data)))
        
        complete_message = bytes(header) + protobuf_data
        padded_message = complete_message + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_message))
        
        result = deserialize_response(padded_message)
        
        assert isinstance(result, protocol_pb2.PluginConfigurationError)
        assert result.error_type == 5
    
    def test_deserialize_invalid_magic(self):
        """Test that invalid magic number raises error"""
        # Create data with wrong magic number
        bad_data = struct.pack('<H', 0xBEEF) + b'\x01\x00\x00' + b'x' * 10
        bad_data += b'\x00' * (DEFAULT_PACKET_SIZE - len(bad_data))
        
        with pytest.raises(USBCommunicationError, match="Invalid magic number"):
            deserialize_response(bad_data)
    
    def test_deserialize_unknown_message_type(self):
        """Test that unknown message type ID raises error"""
        # Create data with unknown message type
        header = bytes([MESSAGE_MAGIC]) + struct.pack('<H', 0xFF) + struct.pack('<H', 5)
        data = header + b'hello' + b'\x00' * (DEFAULT_PACKET_SIZE - len(header) - 5)
        
        with pytest.raises(USBCommunicationError, match="No protobuf handler for message type ID"):
            deserialize_response(data)
    
    def test_deserialize_data_too_short(self):
        """Test that data shorter than header size raises error"""
        with pytest.raises(USBCommunicationError, match="Data too short"):
            deserialize_response(b'\x00\x01')
    
    def test_deserialize_insufficient_data(self):
        """Test that insufficient data for declared length raises error"""
        # Create header claiming 20 bytes but provide less (stay within packet size)
        header = bytes([MESSAGE_MAGIC]) + struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginData) + struct.pack('<H', 20)
        data = header + b'short'  # Only 5 bytes when we claimed 20
        
        with pytest.raises(USBCommunicationError, match="Insufficient data"):
            deserialize_response(data)


class TestProtobufRoundTrip:
    """Test complete round-trip serialization/deserialization"""
    
    def test_configure_peripheral_round_trip(self):
        """Test complete round-trip for HostCommandConfigurePeripheral"""
        # Create original command
        original_cmd = protocol_pb2.HostCommandConfigurePeripheral()
        original_cmd.name = "RoundTripTest"
        original_cmd.addr = b'\xaa\xbb\xcc\xdd\xee\xff'
        
        # Serialize with protobuf
        serialized = serialize_command(original_cmd)
        
        # Extract the protobuf data from the serialized packet
        length = struct.unpack('<H', serialized[3:5])[0]
        protobuf_data = serialized[MESSAGE_HEADER_SIZE:MESSAGE_HEADER_SIZE + length]
        
        # Deserialize back to protobuf
        recovered_cmd = protocol_pb2.HostCommandConfigurePeripheral()
        recovered_cmd.ParseFromString(protobuf_data)
        
        # Verify they match
        assert recovered_cmd.name == original_cmd.name
        assert recovered_cmd.addr == original_cmd.addr
    
    def test_service_info_response_round_trip(self):
        """Test complete round-trip for service info response flow"""
        # Create a response as it would come from the device
        original_response = protocol_pb2.PluginServiceInfoResponse()
        original_response.service_uuid = 0xabcd
        original_response.characteristic_uuids.extend([0x1111, 0x2222, 0x3333])
        original_response.exists = True
        
        # Serialize as the device would
        protobuf_data = original_response.SerializeToString()
        
        # Create complete message as device would send
        header = bytearray()
        header.append(MESSAGE_MAGIC)
        header.extend(struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginServiceInfoResponse))
        header.extend(struct.pack('<H', len(protobuf_data)))
        
        complete_message = bytes(header) + protobuf_data
        padded_message = complete_message + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_message))
        
        # Deserialize as the host would
        recovered_response = deserialize_response(padded_message)
        
        # Verify they match
        assert recovered_response.service_uuid == original_response.service_uuid
        assert list(recovered_response.characteristic_uuids) == list(original_response.characteristic_uuids)
        assert recovered_response.exists == original_response.exists


class TestProtobufIntegrationWithUSBFunctions:
    """Test protobuf integration with USB communication functions"""
    
    @patch('plugin_host.comms.USBDevice')
    def test_usb_send_command_with_protobuf(self, mock_usb_device_class):
        """Test usb_send_command function with protobuf enabled"""
        mock_device = Mock()
        mock_usb_device_class.return_value = mock_device
        
        # Mock send_data to return the expected number of bytes
        mock_device.send_data.return_value = DEFAULT_PACKET_SIZE
        
        cmd = protocol_pb2.HostCommandGetServiceInfo()
        cmd.uuid = 0x9876
        
        # Send command with protobuf enabled
        result = usb_send_command(mock_device, cmd)
        
        # Verify device.send_data was called with properly serialized protobuf data
        assert result is True
        mock_device.send_data.assert_called_once()
        
        # Get the sent data and verify it's properly formatted
        sent_data = mock_device.send_data.call_args[0][0]
        assert len(sent_data) == DEFAULT_PACKET_SIZE
        
        # Verify magic and type ID
        magic = sent_data[0]
        assert magic == MESSAGE_MAGIC
        type_id = struct.unpack('<H', sent_data[1:3])[0]
        assert type_id == protocol_pb2.MessageTypeId.TypeHostCommandGetServiceInfo
    
    @patch('plugin_host.comms.USBDevice')
    def test_usb_receive_response_with_protobuf(self, mock_usb_device_class):
        """Test usb_receive_response function with protobuf enabled"""
        mock_device = Mock()
        mock_usb_device_class.return_value = mock_device
        
        # Create a mock response as device would send
        response = protocol_pb2.PluginServiceInfoResponse()
        response.service_uuid = 0x4321
        response.characteristic_uuids.extend([0x1010, 0x2020])
        response.exists = False
        
        protobuf_data = response.SerializeToString()
        header = bytearray()
        header.append(MESSAGE_MAGIC)
        header.extend(struct.pack('<H', protocol_pb2.MessageTypeId.TypePluginServiceInfoResponse))
        header.extend(struct.pack('<H', len(protobuf_data)))
        
        complete_message = bytes(header) + protobuf_data
        padded_message = complete_message + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_message))
        
        # Mock device.receive_data to return this data
        mock_device.receive_data.return_value = padded_message
        
        # Receive with protobuf enabled
        result = usb_receive_response(mock_device, protocol_pb2.PluginServiceInfoResponse)
        
        # Verify the result
        assert isinstance(result, protocol_pb2.PluginServiceInfoResponse)
        assert result.service_uuid == 0x4321
        assert list(result.characteristic_uuids) == [0x1010, 0x2020]
        assert result.exists is False
