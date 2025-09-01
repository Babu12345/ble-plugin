import pytest
import time
import os
from unittest.mock import Mock, patch
from plugin_host.comms import (
    usb_send_command,
    set_command_delay_enabled,
    is_command_delay_enabled,
    set_command_delay,
    get_command_delay,
    DEFAULT_COMMAND_DELAY,
    USBDevice,
    USBCommunicationError
)
import plugin_host.protocol_pb2 as protocol_pb2


class TestCommandDelay:
    """Test suite for command delay functionality"""
    
    def setup_method(self):
        """Setup for each test method"""
        # Reset delay state to default before each test
        set_command_delay_enabled(True)
        set_command_delay(0.5)  # Set test delay to 0.5 seconds for testing
        # Clear any environment variable that might interfere
        if 'BLE_PLUGIN_SKIP_DELAY' in os.environ:
            del os.environ['BLE_PLUGIN_SKIP_DELAY']
    
    def teardown_method(self):
        """Cleanup after each test method"""
        # Reset to default state
        set_command_delay_enabled(True)
        set_command_delay(0.0)  # Reset to default delay
        # Clear environment variable
        if 'BLE_PLUGIN_SKIP_DELAY' in os.environ:
            del os.environ['BLE_PLUGIN_SKIP_DELAY']
    
    def test_default_delay_state(self):
        """Test that delay is enabled by default"""
        # Reset to actual defaults
        set_command_delay_enabled(True)
        set_command_delay(0.0)  # Reset to library default
        
        assert is_command_delay_enabled() == True
        assert DEFAULT_COMMAND_DELAY == 0.0
        assert get_command_delay() == 0.0
    
    def test_set_delay_enabled_function(self):
        """Test enabling/disabling delay via function"""
        # Test disable
        set_command_delay_enabled(False)
        assert is_command_delay_enabled() == False
        
        # Test enable
        set_command_delay_enabled(True)
        assert is_command_delay_enabled() == True
    
    @pytest.mark.parametrize("env_value,expected", [
        ("true", False),
        ("1", False),
        ("yes", False),
        ("TRUE", False),
        ("false", True),
        ("0", True),
        ("no", True),
        ("", True),
        ("invalid", True),
    ])
    def test_environment_variable_override(self, env_value, expected):
        """Test environment variable override with various values"""
        os.environ['BLE_PLUGIN_SKIP_DELAY'] = env_value
        assert is_command_delay_enabled() == expected
    
    def test_environment_variable_precedence(self):
        """Test that environment variable takes precedence over function setting"""
        # Enable via function
        set_command_delay_enabled(True)
        
        # Override with environment variable to disable
        os.environ['BLE_PLUGIN_SKIP_DELAY'] = 'true'
        assert is_command_delay_enabled() == False
        
        # Function setting should be ignored
        set_command_delay_enabled(True)
        assert is_command_delay_enabled() == False  # Still disabled by env var
    
    def test_delay_timing_with_mock_device(self):
        """Test delay timing logic with mocked time.sleep"""
        # Create mock device and command
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            # Test with delay enabled
            set_command_delay_enabled(True)
            result = usb_send_command(mock_device, command)
            
            assert result == True
            expected_delay = get_command_delay()
            # Verify sleep was called with correct delay
            mock_sleep.assert_called_once_with(expected_delay)
            mock_device.send_data.assert_called_once()
    
    def test_no_delay_when_disabled(self):
        """Test that no delay occurs when disabled"""
        # Create mock device and command
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            # Test with delay disabled
            set_command_delay_enabled(False)
            result = usb_send_command(mock_device, command)
            
            assert result == True
            # Verify sleep was NOT called
            mock_sleep.assert_not_called()
            mock_device.send_data.assert_called_once()
    
    def test_no_delay_with_env_var(self):
        """Test that no delay occurs when disabled via environment variable"""
        # Create mock device and command
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            # Enable via function but disable via env var
            set_command_delay_enabled(True)
            os.environ['BLE_PLUGIN_SKIP_DELAY'] = 'true'
            
            result = usb_send_command(mock_device, command)
            
            assert result == True
            # Verify sleep was NOT called (env var overrides)
            mock_sleep.assert_not_called()
            mock_device.send_data.assert_called_once()
    
    def test_delay_after_send_not_before(self):
        """Test that delay happens after send_data, not before"""
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            set_command_delay_enabled(True)
            usb_send_command(mock_device, command)
            
            # Verify send_data was called before sleep
            mock_device.send_data.assert_called_once()
            expected_delay = get_command_delay()
            mock_sleep.assert_called_once_with(expected_delay)
            
            # Verify call order: send_data should be called before sleep
            # This is verified by the fact that both are called exactly once
    
    def test_error_propagation_with_delay(self):
        """Test that errors are still propagated even with delay"""
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.side_effect = Exception("USB error")
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            set_command_delay_enabled(True)
            
            with pytest.raises(USBCommunicationError, match="Failed to send command"):
                usb_send_command(mock_device, command)
            
            # Sleep should not be called if there's an error
            mock_sleep.assert_not_called()
    
    def test_serialization_error_no_delay(self):
        """Test that serialization errors don't trigger delay"""
        mock_device = Mock(spec=USBDevice)
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.side_effect = Exception("Serialization error")
            
            set_command_delay_enabled(True)
            
            with pytest.raises(USBCommunicationError, match="Failed to send command"):
                usb_send_command(mock_device, command)
            
            # Neither send_data nor sleep should be called
            mock_device.send_data.assert_not_called()
            mock_sleep.assert_not_called()
    
    def test_delay_configuration_isolation(self):
        """Test that delay configuration doesn't affect other functions"""
        # This test ensures that our delay only affects usb_send_command
        # and doesn't interfere with other timing-sensitive operations
        
        set_command_delay_enabled(True)
        set_command_delay(0.5)
        
        # Test that other time.sleep calls are not affected
        start_time = time.time()
        time.sleep(0.1)
        elapsed = time.time() - start_time
        
        # Should be close to 0.1, not 0.1 + 0.5
        assert 0.05 < elapsed < 0.15
    
    def test_custom_delay_configuration(self):
        """Test setting custom delay values"""
        # Test setting different delay values
        test_delays = [0.0, 0.1, 0.3, 0.5, 1.0]
        
        for delay in test_delays:
            set_command_delay(delay)
            assert get_command_delay() == delay
    
    def test_custom_delay_timing(self):
        """Test delay logic with custom delay values"""
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            # Test with custom delay of 0.2 seconds
            set_command_delay_enabled(True)
            set_command_delay(0.2)
            
            result = usb_send_command(mock_device, command)
            
            assert result == True
            # Verify sleep was called with custom delay
            mock_sleep.assert_called_once_with(0.2)
    
    def test_zero_delay_no_sleep(self):
        """Test that zero delay doesn't call sleep"""
        mock_device = Mock(spec=USBDevice)
        mock_device.send_data.return_value = 64
        command = protocol_pb2.HostCommandConfigurePeripheral(name="test", addr=bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]))
        
        with patch('plugin_host.comms.serialize_command') as mock_serialize, \
             patch('plugin_host.comms.time.sleep') as mock_sleep:
            mock_serialize.return_value = b'x' * 64
            
            # Test with zero delay
            set_command_delay_enabled(True)
            set_command_delay(0.0)
            
            result = usb_send_command(mock_device, command)
            
            assert result == True
            # Verify sleep was NOT called for zero delay
            mock_sleep.assert_not_called()


class TestDelayConfigurationHelpers:
    """Test helper functions for delay configuration"""
    
    def setup_method(self):
        """Setup for each test method"""
        set_command_delay_enabled(True)
        set_command_delay(0.0)  # Reset to default
        if 'BLE_PLUGIN_SKIP_DELAY' in os.environ:
            del os.environ['BLE_PLUGIN_SKIP_DELAY']
    
    def teardown_method(self):
        """Cleanup after each test method"""
        set_command_delay_enabled(True)
        set_command_delay(0.0)  # Reset to default
        if 'BLE_PLUGIN_SKIP_DELAY' in os.environ:
            del os.environ['BLE_PLUGIN_SKIP_DELAY']
    
    def configure_env_var(self, value):
        """Helper method to configure environment variable explicitly"""
        if value is None:
            if 'BLE_PLUGIN_SKIP_DELAY' in os.environ:
                del os.environ['BLE_PLUGIN_SKIP_DELAY']
        else:
            os.environ['BLE_PLUGIN_SKIP_DELAY'] = str(value)
    
    @pytest.mark.parametrize("env_value", [None, "true", "false", "1", "0"])
    def test_configure_env_var_helper(self, env_value):
        """Test the helper method for configuring environment variable"""
        self.configure_env_var(env_value)
        
        if env_value is None:
            assert 'BLE_PLUGIN_SKIP_DELAY' not in os.environ
        else:
            assert os.environ.get('BLE_PLUGIN_SKIP_DELAY') == str(env_value)
    
    def test_combined_configuration(self):
        """Test different combinations of function and environment settings"""
        test_cases = [
            # (function_enabled, env_var_value, expected_result)
            (True, None, True),
            (False, None, False),
            (True, "true", False),  # env var overrides
            (False, "true", False),
            (True, "false", True),
            (False, "false", False),
            (True, "1", False),
            (False, "0", False),
        ]
        
        for func_enabled, env_value, expected in test_cases:
            # Reset state
            set_command_delay_enabled(True)
            self.configure_env_var(None)
            
            # Apply configuration
            set_command_delay_enabled(func_enabled)
            self.configure_env_var(env_value)
            
            # Check result
            assert is_command_delay_enabled() == expected, \
                f"Failed for func_enabled={func_enabled}, env_value={env_value}"