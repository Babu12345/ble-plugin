import pytest
from unittest.mock import Mock, patch
from plugin_host.comms import USBHostDevice, USBCommunicationError
from plugin_host.types import (
    HostCommandConfigurePeripheral,
    HostCommandGetServiceInfo,
    PluginServiceInfoResponse,
    PluginCharacteristicInfoResponse,
    BLEProperties,
    BluetoothAddressType
)

class TestUSBHostDevice:
    """Test suite for USBHostDevice class"""
    
    def setup_method(self):
        """Setup for each test method"""
        self.host_device = USBHostDevice()
    
    def test_initialization(self) -> None:
        """Test USBHostDevice initialization"""
        assert not self.host_device.is_connected()
        assert self.host_device.usb_device is not None
    
    @patch('plugin_host.comms.USBDevice.connect')
    def test_connect_success(self, mock_connect) -> None:
        """Test successful connection"""
        mock_connect.return_value = True
        
        result = self.host_device.connect()
        
        assert result is True
        assert self.host_device.is_connected()
        mock_connect.assert_called_once()
    
    @patch('plugin_host.comms.USBDevice.connect')
    def test_connect_failure(self, mock_connect) -> None:
        """Test failed connection"""
        mock_connect.return_value = False
        
        result = self.host_device.connect()
        
        assert result is False
        assert not self.host_device.is_connected()
    
    @patch('plugin_host.comms.USBDevice.disconnect')
    def test_disconnect(self, mock_disconnect) -> None:
        """Test disconnection"""
        # First connect
        self.host_device._connected = True
        
        self.host_device.disconnect()
        
        assert not self.host_device.is_connected()
        mock_disconnect.assert_called_once()
    
    @patch('plugin_host.comms.usb_send_command')
    def test_configure_peripheral(self, mock_send) -> None:
        """Test configure_peripheral method"""
        name = "TestDevice"
        uuid = "12345678-1234-1234-1234-123456789abc"
        
        self.host_device.configure_peripheral(name, uuid)
        
        mock_send.assert_called_once()
        args, kwargs = mock_send.call_args
        command = args[1]  # Second argument is the command
        assert isinstance(command, HostCommandConfigurePeripheral)
        assert command.name == name
        assert command.uuid == uuid
    
    @patch('plugin_host.comms.usb_send_and_receive')
    def test_get_service_info(self, mock_send_receive) -> None:
        """Test get_service_info method"""
        uuid = "87654321-4321-4321-4321-cba987654321"
        expected_response = PluginServiceInfoResponse(
            service_uuid=uuid,
            characteristic_uuids=["char1", "char2"],
            exists=True
        )
        mock_send_receive.return_value = expected_response
        
        result = self.host_device.get_service_info(uuid)
        
        assert result == expected_response
        mock_send_receive.assert_called_once()
        args, kwargs = mock_send_receive.call_args
        command = args[1]  # Second argument is the command
        response_type = args[2]  # Third argument is response type
        assert isinstance(command, HostCommandGetServiceInfo)
        assert command.uuid == uuid
        assert response_type == PluginServiceInfoResponse
    
    @patch('plugin_host.comms.usb_send_command')
    def test_configure_characteristic(self, mock_send) -> None:
        """Test configure_characteristic method"""
        uuid = "char-uuid"
        service_uuid = "service-uuid"
        properties = [BLEProperties.READ, BLEProperties.WRITE]
        
        self.host_device.configure_characteristic(uuid, service_uuid, properties)
        
        mock_send.assert_called_once()
        args, kwargs = mock_send.call_args
        command = args[1]
        assert command.uuid == uuid
        assert command.service_uuid == service_uuid
        assert command.properties == properties
    
    @patch('plugin_host.comms.usb_send_command')
    def test_start_advertisement(self, mock_send) -> None:
        """Test start_advertisement method"""
        self.host_device.start_advertisement(allow_multi_connect=True)
        
        mock_send.assert_called_once()
        args, kwargs = mock_send.call_args
        command = args[1]
        assert command.allow_multi_connect is True
    
    @patch('plugin_host.comms.usb_send_command')
    def test_notify_characteristic_value(self, mock_send) -> None:
        """Test notify_characteristic_value method"""
        address = b'\x12\x34\x56\x78\x9a\xbc'
        address_type = BluetoothAddressType.Public
        char_uuid = "char-uuid"
        service_uuid = "service-uuid"
        value = b"test_value"
        
        self.host_device.notify_characteristic_value(
            address, address_type, char_uuid, service_uuid, value
        )
        
        mock_send.assert_called_once()
        args, kwargs = mock_send.call_args
        command = args[1]
        assert command.address == address
        assert command.address_type == address_type
        assert command.characteristic_uuid == char_uuid
        assert command.service_uuid == service_uuid
        assert command.value == value
    
    @patch('plugin_host.comms.usb_send_command')
    def test_send_command_generic(self, mock_send) -> None:
        """Test generic send_command method"""
        cmd = HostCommandGetServiceInfo(uuid="test-uuid")
        
        self.host_device.send_command(cmd)
        
        mock_send.assert_called_once_with(self.host_device.usb_device, cmd)
    
    @patch('plugin_host.comms.usb_receive_response')
    def test_receive_response_generic(self, mock_receive) -> None:
        """Test generic receive_response method"""
        expected_response = PluginServiceInfoResponse(
            service_uuid="test",
            characteristic_uuids=[],
            exists=False
        )
        mock_receive.return_value = expected_response
        
        result = self.host_device.receive_response(PluginServiceInfoResponse)
        
        assert result == expected_response
        mock_receive.assert_called_once_with(self.host_device.usb_device, PluginServiceInfoResponse)
    
    def test_context_manager(self) -> None:
        """Test context manager functionality"""
        with patch.object(self.host_device, 'connect') as mock_connect, \
             patch.object(self.host_device, 'disconnect') as mock_disconnect:
            
            with self.host_device as device:
                assert device is self.host_device
                mock_connect.assert_called_once()
            
            mock_disconnect.assert_called_once()
    
    @patch('plugin_host.comms.usb_send_command')
    def test_error_propagation(self, mock_send) -> None:
        """Test that USB communication errors are properly propagated"""
        mock_send.side_effect = USBCommunicationError("Test error")
        
        with pytest.raises(USBCommunicationError, match="Test error"):
            self.host_device.configure_peripheral("test", "test-uuid")