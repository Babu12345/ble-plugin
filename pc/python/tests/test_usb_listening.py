import pytest
import time
import threading
import uuid as uuid_module
from unittest.mock import Mock, patch, MagicMock
from plugin_host.comms import (
    MessageDecoder,
    USBDataListener,
    USBMessageHandler,
    USBHostDevice,
    USBCommunicationError,
    serialize_command,
)
import plugin_host.protocol_pb2 as protocol_pb2

class TestMessageDecoder:
    """Test suite for MessageDecoder class"""
    
    def test_decode_plugin_data(self) -> None:
        """Test decoding PluginData messages"""
        # Create a test PluginData message
        original_message = protocol_pb2.PluginData(
            src_addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]), src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.NotifyType,
            characteristic_uuid=0x2A19,  # Battery Level characteristic
            service_uuid=0x180F,  # Battery Service
            data=b"test_data"
        )
        
        # Serialize it
        serialized = serialize_command(original_message)
        
        # Decode it
        decoded = MessageDecoder.decode_message(serialized)
        
        assert decoded is not None
        assert isinstance(decoded, protocol_pb2.PluginData)
        assert decoded.src_addr == original_message.src_addr
        assert decoded.src_addr_type == original_message.src_addr_type
        assert decoded.data == original_message.data
        assert decoded.send_type == original_message.send_type
    
    def test_decode_service_info_response(self) -> None:
        """Test decoding PluginServiceInfoResponse messages"""
        original_message = protocol_pb2.PluginServiceInfoResponse(
            service_uuid=0x1234,
            characteristic_uuids=[
                0x1234
            ],
            exists=True
        )
        
        serialized = serialize_command(original_message)
        decoded = MessageDecoder.decode_message(serialized)
        
        assert decoded is not None
        assert isinstance(decoded, protocol_pb2.PluginServiceInfoResponse)
        assert decoded.service_uuid == original_message.service_uuid
        assert decoded.characteristic_uuids == original_message.characteristic_uuids
        assert decoded.exists == original_message.exists
    
    def test_decode_unknown_message(self) -> None:
        """Test decoding unknown/invalid messages"""
        # Invalid data that can't be decoded
        invalid_data = b"this is not a valid message"
        
        decoded = MessageDecoder.decode_message(invalid_data)
        
        assert decoded is None
    
    def test_get_message_type_name(self) -> None:
        """Test getting message type names"""
        message = protocol_pb2.PluginData(
            src_addr=bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05]), src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.ReadType,
            characteristic_uuid=0x2A00,  # Device Name characteristic
            service_uuid=0x1800,  # Generic Access Service
            data=b"data"
        )
        
        type_name = MessageDecoder.get_message_type_name(message)
        
        assert type_name == "PluginData"


class TestUSBDataListener:
    """Test suite for USBDataListener class"""
    
    def setup_method(self):
        """Setup for each test method"""
        self.mock_host = Mock(spec=USBHostDevice)
        self.mock_host.is_connected.return_value = True
        self.mock_host.usb_device = Mock()
        self.listener = USBDataListener(self.mock_host)
    
    def test_initialization(self) -> None:
        """Test USBDataListener initialization"""
        assert self.listener.host_device == self.mock_host
        assert self.listener.receive_timeout_ms == 500
        assert not self.listener.is_listening()
        assert not self.listener.has_messages()
    
    def test_start_listening_not_connected(self) -> None:
        """Test starting listener when device is not connected"""
        self.mock_host.is_connected.return_value = False
        
        with pytest.raises(USBCommunicationError, match="Host device must be connected"):
            self.listener.start_listening()
    
    def test_start_listening_success(self) -> None:
        """Test successful listener start"""
        result = self.listener.start_listening()
        
        assert result is True
        assert self.listener.is_listening()
        
        # Clean up
        self.listener.stop_listening()
    
    def test_start_listening_already_running(self) -> None:
        """Test starting listener when already running"""
        self.listener.start_listening()
        
        # Try to start again
        result = self.listener.start_listening()
        
        assert result is False
        
        # Clean up
        self.listener.stop_listening()
    
    def test_stop_listening(self) -> None:
        """Test stopping the listener"""
        self.listener.start_listening()
        assert self.listener.is_listening()
        
        result = self.listener.stop_listening()
        
        assert result is True
        assert not self.listener.is_listening()
    
    def test_stop_listening_not_running(self) -> None:
        """Test stopping listener when not running"""
        result = self.listener.stop_listening()
        
        assert result is False
    
    def test_message_queue_operations(self) -> None:
        """Test message queue operations"""
        # Test empty queue
        assert not self.listener.has_messages()
        assert self.listener.get_message_nowait() is None
        assert self.listener.get_message(timeout=0.1) is None
        
        # Manually add a message to test queue operations
        test_message = {
            'timestamp': time.time(),
            'message_type': 'Test',
            'message': None,
            'raw_data': b'test',
            'decoded': False
        }
        self.listener.message_queue.put(test_message)
        
        # Test queue operations
        assert self.listener.has_messages()
        
        message = self.listener.get_message_nowait()
        assert message == test_message
        
        assert not self.listener.has_messages()
    
    def test_clear_messages(self) -> None:
        """Test clearing messages from queue"""
        # Add some test messages
        for i in range(3):
            self.listener.message_queue.put(f"message_{i}")
        
        assert self.listener.has_messages()
        
        cleared_count = self.listener.clear_messages()
        
        assert cleared_count == 3
        assert not self.listener.has_messages()
    
    def test_stats_tracking(self) -> None:
        """Test statistics tracking"""
        initial_stats = self.listener.get_stats()
        
        assert initial_stats['messages_received'] == 0
        assert initial_stats['decode_successes'] == 0
        assert initial_stats['decode_failures'] == 0
        assert initial_stats['usb_errors'] == 0
        assert initial_stats['queue_size'] == 0
        assert initial_stats['is_listening'] is False
        
        # Reset stats
        self.listener.reset_stats()
        stats_after_reset = self.listener.get_stats()
        
        assert stats_after_reset == initial_stats
    
    @patch('time.sleep')  # Speed up the test
    def test_listen_loop_with_valid_message(self, mock_sleep) -> None:
        """Test the listening loop with valid messages"""
        # Create a valid message
        test_message = protocol_pb2.PluginData(
            src_addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]), src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.NotifyType,
            characteristic_uuid=0x2A05,  # Service Changed characteristic
            service_uuid=0x1801,  # Generic Attribute Service
            data=b"test"
        )
        serialized = serialize_command(test_message)
        
        # Mock USB device to return the message once, then timeout
        self.mock_host.usb_device.receive_data.side_effect = [
            serialized,  # First call returns valid data
            USBCommunicationError("timeout")  # Second call times out
        ]
        
        # Start listener
        self.listener.start_listening()
        
        # Wait a bit for message processing
        time.sleep(0.1)
        
        # Stop listener
        self.listener.stop_listening()
        
        # Check that message was processed
        stats = self.listener.get_stats()
        assert stats['messages_received'] == 1
        assert stats['decode_successes'] == 1
        
        # Check that message is in queue
        assert self.listener.has_messages()
        message_info = self.listener.get_message_nowait()
        assert message_info['decoded'] is True
        assert isinstance(message_info['message'], protocol_pb2.PluginData)


class TestUSBMessageHandler:
    """Test suite for USBMessageHandler class"""
    
    def setup_method(self):
        """Setup for each test method"""
        self.handler = USBMessageHandler()
    
    def test_initialization(self) -> None:
        """Test USBMessageHandler initialization"""
        assert len(self.handler.message_callbacks) == 0
        assert len(self.handler.message_filters) == 0
        assert len(self.handler.message_stats) == 0
        assert self.handler.global_callback is None
    
    def test_register_callback(self) -> None:
        """Test registering message callbacks"""
        def test_callback(message, info):
            pass
        
        self.handler.register_callback(protocol_pb2.PluginData, test_callback)
        
        assert protocol_pb2.PluginData in self.handler.message_callbacks
        assert self.handler.message_callbacks[protocol_pb2.PluginData] == test_callback
    
    def test_register_filter(self) -> None:
        """Test registering message filters"""
        def test_filter(message, info):
            return True
        
        self.handler.register_filter(protocol_pb2.PluginData, test_filter)
        
        assert protocol_pb2.PluginData in self.handler.message_filters
        assert self.handler.message_filters[protocol_pb2.PluginData] == test_filter
    
    def test_set_global_callback(self) -> None:
        """Test setting global callback"""
        def global_callback(message, info):
            pass
        
        self.handler.set_global_callback(global_callback)
        
        assert self.handler.global_callback == global_callback
    
    def test_handle_decoded_message_with_callback(self) -> None:
        """Test handling decoded message with callback"""
        callback_called = False
        received_message = None
        received_info = None
        
        def test_callback(message, info):
            nonlocal callback_called, received_message, received_info
            callback_called = True
            received_message = message
            received_info = info
        
        self.handler.register_callback(protocol_pb2.PluginData, test_callback)
        
        test_message = protocol_pb2.PluginData(
            src_addr=bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05]), src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.WriteType,
            characteristic_uuid=0x2A37,  # Heart Rate Measurement characteristic
            service_uuid=0x180D,  # Heart Rate Service
            data=b"test_data"
        )
        
        message_info = {
            'timestamp': time.time(),
            'message_type': 'PluginData',
            'message': test_message,
            'raw_data': b'raw',
            'decoded': True
        }
        
        result = self.handler.handle_message(message_info)
        
        assert result is True
        assert callback_called is True
        assert received_message == test_message
        assert received_info == message_info
        
        # Check stats
        stats = self.handler.get_stats()
        assert stats['PluginData'] == 1
    
    def test_handle_message_with_filter(self) -> None:
        """Test handling message with filter"""
        callback_called = False
        
        def test_callback(message, info):
            nonlocal callback_called
            callback_called = True
        
        def test_filter(message, info):
            # Filter out messages with specific src_addr
            return message.src_addr != bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
        
        self.handler.register_callback(protocol_pb2.PluginData, test_callback)
        self.handler.register_filter(protocol_pb2.PluginData, test_filter)
        
        # Test message that should be filtered out
        filtered_message = protocol_pb2.PluginData(
            src_addr=bytes([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]), src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.ReadType,
            characteristic_uuid=0x2A01,  # Appearance characteristic
            service_uuid=0x1800,  # Generic Access Service
            data=b"data"
        )
        
        filtered_info = {
            'message_type': 'PluginData',
            'message': filtered_message,
            'decoded': True
        }
        
        result = self.handler.handle_message(filtered_info)
        
        assert result is False  # Message was filtered out
        assert callback_called is False
        
        # Test message that should pass through
        callback_called = False
        passed_message = protocol_pb2.PluginData(
            src_addr=bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05]),
            src_addr_type=protocol_pb2.BluetoothAddressType.Public,
            send_type=protocol_pb2.PluginDataSendType.ReadType,
            characteristic_uuid=0x2A01,  # Appearance characteristic
            service_uuid=0x1800,  # Generic Access Service
            data=b"data"
        )
        
        passed_info = {
            'message_type': 'PluginData',
            'message': passed_message,
            'decoded': True
        }
        
        result = self.handler.handle_message(passed_info)
        
        assert result is True
        assert callback_called is True
    
    def test_handle_unknown_message(self) -> None:
        """Test handling unknown/undecoded messages"""
        global_callback_called = False
        received_message = None
        
        def global_callback(message, info):
            nonlocal global_callback_called, received_message
            global_callback_called = True
            received_message = message
        
        self.handler.set_global_callback(global_callback)
        
        unknown_info = {
            'message_type': 'Unknown',
            'message': None,
            'raw_data': b'unknown_data',
            'decoded': False
        }
        
        result = self.handler.handle_message(unknown_info)
        
        assert result is False
        assert global_callback_called is True
        assert received_message is None
    
    def test_stats_operations(self) -> None:
        """Test statistics operations"""
        # Process some messages
        for i in range(3):
            message_info = {
                'message_type': 'PluginData',
                'message': protocol_pb2.PluginData(src_addr=bytes([0x00, 0x01, 0x02, 0x03, 0x04, i % 256]), src_addr_type=protocol_pb2.BluetoothAddressType.Public, send_type=protocol_pb2.PluginDataSendType.ReadType, characteristic_uuid=0x2A00, service_uuid=0x1800, data=b"data"),
                'decoded': True
            }
            self.handler.handle_message(message_info)
        
        # Process different message type
        service_info = {
            'message_type': 'PluginServiceInfoResponse',
            'message': protocol_pb2.PluginServiceInfoResponse(service_uuid=0x1234, characteristic_uuids=[], exists=True),
            'decoded': True
        }
        self.handler.handle_message(service_info)
        
        stats = self.handler.get_stats()
        
        assert stats['PluginData'] == 3
        assert stats['PluginServiceInfoResponse'] == 1
        
        # Reset stats
        self.handler.reset_stats()
        new_stats = self.handler.get_stats()
        
        assert len(new_stats) == 0