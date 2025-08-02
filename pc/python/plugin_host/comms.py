import attrs2bin
import usb.core
import usb.util
import struct
import threading
import time
import queue
from typing import Any, Optional
from plugin_host.types import *

# Communicate between the host (PC) and the usb plugin

# USB Configuration
USB_VENDOR_ID = 0x1234
USB_PRODUCT_ID = 0x5678
USB_ENDPOINT_OUT = 0x01
USB_ENDPOINT_IN = 0x81
USB_TIMEOUT_MS = 1000
DEFAULT_PACKET_SIZE = 256

# Protocol Configuration
DATA_BYTES_LENGTH_IN_BYTES = 2  # First 2 bytes contain the length of the serialized data

class USBCommunicationError(Exception):
    """Exception for USB communication errors"""
    pass

class USBDevice:
    """Handles USB communication with the plugin device"""
    
    def __init__(self, vendor_id: int = USB_VENDOR_ID, product_id: int = USB_PRODUCT_ID):
        self.vendor_id = vendor_id
        self.product_id = product_id
        self.device = None
        self.endpoint_out = USB_ENDPOINT_OUT
        self.endpoint_in = USB_ENDPOINT_IN
        
    def connect(self) -> bool:
        """Connect to the USB device"""
        try:
            self.device = usb.core.find(idVendor=self.vendor_id, idProduct=self.product_id)
            if self.device is None:
                raise USBCommunicationError(f"Device not found (VID: 0x{self.vendor_id:04x}, PID: 0x{self.product_id:04x})")
            
            # Set the active configuration
            self.device.set_configuration()
            return True
        except Exception as e:
            raise USBCommunicationError(f"Failed to connect to USB device: {e}")
    
    def disconnect(self):
        """Disconnect from the USB device"""
        if self.device:
            usb.util.dispose_resources(self.device)
            self.device = None

    def send_data(self, data: bytes, timeout: int = USB_TIMEOUT_MS) -> int:
        """Send raw bytes to the USB device"""
        if not self.device:
            raise USBCommunicationError("Device not connected")
        
        try:
            return self.device.write(self.endpoint_out, data, timeout)
        except Exception as e:
            raise USBCommunicationError(f"Failed to send data: {e}")
    
    def receive_data(self, size: int = DEFAULT_PACKET_SIZE, timeout: int = USB_TIMEOUT_MS) -> bytes:
        """Receive raw bytes from the USB device"""
        if not self.device:
            raise USBCommunicationError("Device not connected")
        
        try:
            data = self.device.read(self.endpoint_in, size, timeout)
            return bytes(data)
        except Exception as e:
            raise USBCommunicationError(f"Failed to receive data: {e}")

def serialize_command(command: Any) -> bytes:
    """
    Serialize a protocol command object to bytes using attrs2bin with length prefix
    
    Format: [2-byte little-endian length][serialized data][padding to DEFAULT_PACKET_SIZE]
    
    Args:
        command: Any command object with attr.s decoration
        
    Returns:
        bytes: Serialized command data with length prefix and padding
        
    Raises:
        USBCommunicationError: If serialization fails
    """
    try:
        # Use attrs2bin to serialize the command
        serialized_data = attrs2bin.serialize(command)
        data_length = len(serialized_data)
        
        # Ensure the data fits within the packet size (accounting for length prefix)
        if data_length + DATA_BYTES_LENGTH_IN_BYTES > DEFAULT_PACKET_SIZE:
            raise USBCommunicationError(f"Command size ({data_length}) + length prefix exceeds packet size ({DEFAULT_PACKET_SIZE})")
        
        # Create length prefix (2-byte little-endian)
        length_prefix = struct.pack('<H', data_length)
        
        # Combine length prefix with serialized data
        complete_data = length_prefix + serialized_data
        
        # Pad to packet size for consistent USB transfers
        padded_data = complete_data + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_data))
        return padded_data
        
    except Exception as e:
        raise USBCommunicationError(f"Failed to serialize command: {e}")

def deserialize_response(data: bytes, response_type: type) -> Any:
    """
    Deserialize bytes to a protocol response object using attrs2bin with length prefix
    
    Format: [2-byte little-endian length][serialized data][padding]
    
    Args:
        data: Raw bytes received from USB
        response_type: The expected response type class
        
    Returns:
        Any: Deserialized response object
        
    Raises:
        USBCommunicationError: If deserialization fails
    """
    try:
        # Extract length from first 2 bytes (little-endian)
        if len(data) < DATA_BYTES_LENGTH_IN_BYTES:
            raise USBCommunicationError(f"Data too short to contain length prefix: {len(data)} bytes")
        
        data_length = struct.unpack('<H', data[:DATA_BYTES_LENGTH_IN_BYTES])[0]
        
        # Validate length
        if data_length > DEFAULT_PACKET_SIZE:
            raise USBCommunicationError(f"Data length ({data_length}) exceeds packet size ({DEFAULT_PACKET_SIZE})")
        
        # Extract the actual serialized data using the length
        start_idx = DATA_BYTES_LENGTH_IN_BYTES
        end_idx = start_idx + data_length
        
        if end_idx > len(data):
            raise USBCommunicationError(f"Insufficient data: expected {data_length} bytes, got {len(data) - DATA_BYTES_LENGTH_IN_BYTES}")
        
        serialized_data = data[start_idx:end_idx]
        
        # Use attrs2bin to deserialize the response
        response = attrs2bin.deserialize(serialized_data, response_type)
        return response
        
    except Exception as e:
        raise USBCommunicationError(f"Failed to deserialize response: {e}")

def usb_send_command(device: USBDevice, command: Any) -> bool:
    """
    Send a protocol command over USB
    
    Args:
        device: Connected USB device
        command: Protocol command object to send
        
    Returns:
        bool: True if successful
        
    Raises:
        USBCommunicationError: If sending fails
    """
    try:
        serialized_data = serialize_command(command)
        bytes_sent = device.send_data(serialized_data)
        return bytes_sent == len(serialized_data)
    except Exception as e:
        raise USBCommunicationError(f"Failed to send command: {e}")

def usb_receive_response(device: USBDevice, response_type: type) -> Any:
    """
    Receive and deserialize a protocol response over USB
    
    Args:
        device: Connected USB device
        response_type: Expected response type class
        
    Returns:
        Any: Deserialized response object
        
    Raises:
        USBCommunicationError: If receiving fails
    """
    try:
        raw_data = device.receive_data()
        response = deserialize_response(raw_data, response_type)
        return response
    except Exception as e:
        raise USBCommunicationError(f"Failed to receive response: {e}")

def usb_send_and_receive(device: USBDevice, command: Any, response_type: type) -> Any:
    """
    Send a command and receive response in one operation
    
    Args:
        device: Connected USB device
        command: Protocol command object to send
        response_type: Expected response type class
        
    Returns:
        Any: Deserialized response object
        
    Raises:
        USBCommunicationError: If communication fails
    """
    usb_send_command(device, command)
    return usb_receive_response(device, response_type)


class USBHostDevice:
    """
    High-level USB Host Device class that automatically handles serialization/deserialization
    of protocol commands and responses for communication with BLE plugin devices.
    
    This class provides a convenient interface for sending host commands and receiving
    plugin responses with automatic protocol handling.
    """
    
    def __init__(self, vendor_id: int = USB_VENDOR_ID, product_id: int = USB_PRODUCT_ID):
        """
        Initialize the USB Host Device
        
        Args:
            vendor_id: USB vendor ID of the plugin device
            product_id: USB product ID of the plugin device
        """
        self.usb_device = USBDevice(vendor_id, product_id)
        self._connected = False
    
    def connect(self) -> bool:
        """
        Connect to the USB plugin device
        
        Returns:
            bool: True if connection successful
            
        Raises:
            USBCommunicationError: If connection fails
        """
        result = self.usb_device.connect()
        self._connected = result
        return result
    
    def disconnect(self) -> None:
        """Disconnect from the USB plugin device"""
        self.usb_device.disconnect()
        self._connected = False
    
    def is_connected(self) -> bool:
        """Check if device is connected"""
        return self._connected
    
    # Host Command Methods
    
    def configure_peripheral(self, name: str, uuid: str) -> None:
        """
        Configure a peripheral device
        
        Args:
            name: Peripheral name (max 32 characters)
            uuid: Peripheral UUID as string
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandConfigurePeripheral(name=name, uuid=uuid)
        usb_send_command(self.usb_device, cmd)
    
    def configure_service(self, uuid: str) -> None:
        """
        Configure a service
        
        Args:
            uuid: Service UUID as string
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandConfigureService(uuid=uuid)
        usb_send_command(self.usb_device, cmd)
    
    def configure_characteristic(self, uuid: str, service_uuid: str, properties: list) -> None:
        """
        Configure a characteristic
        
        Args:
            uuid: Characteristic UUID as string
            service_uuid: Service UUID this characteristic belongs to
            properties: List of BLE properties
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandConfigureCharacteristic(
            uuid=uuid,
            service_uuid=service_uuid,
            properties=properties
        )
        usb_send_command(self.usb_device, cmd)
    
    def configure_characteristic_read(self, uuid: str, service_uuid: str, value: bytes) -> None:
        """
        Configure characteristic read operation
        
        Args:
            uuid: Characteristic UUID as string
            service_uuid: Service UUID this characteristic belongs to
            value: Read value as bytes (max 32 bytes)
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandConfigureCharacteristicRead(
            uuid=uuid,
            service_uuid=service_uuid,
            value=value
        )
        usb_send_command(self.usb_device, cmd)
    
    def get_service_info(self, uuid: str) -> PluginServiceInfoResponse:
        """
        Get service information
        
        Args:
            uuid: Service UUID as string
            
        Returns:
            PluginServiceInfoResponse: Service information response
            
        Raises:
            USBCommunicationError: If communication fails
        """
        cmd = HostCommandGetServiceInfo(uuid=uuid)
        return usb_send_and_receive(self.usb_device, cmd, PluginServiceInfoResponse)
    
    def get_characteristic_info(self, characteristic_uuid: str, service_uuid: str) -> PluginCharacteristicInfoResponse:
        """
        Get characteristic information
        
        Args:
            characteristic_uuid: Characteristic UUID as string
            service_uuid: Service UUID this characteristic belongs to
            
        Returns:
            PluginCharacteristicInfoResponse: Characteristic information response
            
        Raises:
            USBCommunicationError: If communication fails
        """
        cmd = HostCommandGetCharacteristicInfo(
            characteristic_uuid=characteristic_uuid,
            service_uuid=service_uuid
        )
        return usb_send_and_receive(self.usb_device, cmd, PluginCharacteristicInfoResponse)
    
    def start_advertisement(self, allow_multi_connect: bool = False) -> None:
        """
        Start advertisement
        
        Args:
            allow_multi_connect: Allow multiple central connections
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandStartAdvertisement(allow_multi_connect=allow_multi_connect)
        usb_send_command(self.usb_device, cmd)
    
    def notify_characteristic_value(self, address: bytes, address_type: BluetoothAddressType, 
                                  characteristic_uuid: str, service_uuid: str, value: bytes) -> None:
        """
        Notify characteristic value
        
        Args:
            address: Device Address as 6-byte array
            address_type: Address type
            characteristic_uuid: Characteristic UUID as string
            service_uuid: Service UUID this characteristic belongs to
            value: Value to notify as bytes (max 32 bytes)
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = HostCommandNotifyCharacteristicValue(
            address=address,
            address_type=address_type,
            characteristic_uuid=characteristic_uuid,
            service_uuid=service_uuid,
            value=value
        )
        usb_send_command(self.usb_device, cmd)
    
    # Generic sending and receiving methods
    
    def send_command(self, command: Any) -> None:
        """
        Send any protocol command with automatic serialization
        
        Args:
            command: Protocol command object
            
        Raises:
            USBCommunicationError: If sending fails
        """
        usb_send_command(self.usb_device, command)
    
    def receive_response(self, response_type: type) -> Any:
        """
        Receive and deserialize a protocol response
        
        Args:
            response_type: Expected response type class
            
        Returns:
            Any: Deserialized response object
            
        Raises:
            USBCommunicationError: If receiving fails
        """
        return usb_receive_response(self.usb_device, response_type)
    
    def send_and_receive(self, command: Any, response_type: type) -> Any:
        """
        Send a command and receive response with automatic serialization/deserialization
        
        Args:
            command: Protocol command object to send
            response_type: Expected response type class
            
        Returns:
            Any: Deserialized response object
            
        Raises:
            USBCommunicationError: If communication fails
        """
        return usb_send_and_receive(self.usb_device, command, response_type)
    
    # Context manager support
    
    def __enter__(self):
        """Context manager entry - automatically connect"""
        self.connect()
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit - automatically disconnect"""
        self.disconnect()
        return False  # Don't suppress exceptions


class MessageDecoder:
    """Utility class to decode incoming plugin messages by trying different message types"""
    
    # List of possible plugin response/data types to try when decoding
    # Order matters - more specific types first
    PLUGIN_MESSAGE_TYPES = [
        PluginServiceInfoResponse,
        PluginCharacteristicInfoResponse,
        PluginConfigurationError,
        PluginData,  # Most generic, try last
    ]
    
    @classmethod
    def decode_message(cls, raw_data: bytes) -> Optional[Any]:
        """
        Try to decode raw bytes as different plugin message types
        
        Args:
            raw_data: Raw bytes received from USB device
            
        Returns:
            Decoded message object or None if decoding failed
        """
        for message_type in cls.PLUGIN_MESSAGE_TYPES:
            try:
                decoded = deserialize_response(raw_data, message_type)
                return decoded
            except Exception:
                # Try next message type
                continue
        
        return None
    
    @classmethod
    def get_message_type_name(cls, message: Any) -> str:
        """Get human-readable name for message type"""
        return type(message).__name__


class USBDataListener:
    """
    Thread-safe USB data listener that continuously monitors for incoming data
    
    This class runs a background thread that listens for incoming USB data,
    automatically decodes message types, and queues them for processing.
    """
    
    def __init__(self, host_device: USBHostDevice, receive_timeout_ms: int = 500):
        """
        Initialize the USB data listener
        
        Args:
            host_device: Connected USBHostDevice instance
            receive_timeout_ms: Timeout for USB receive operations in milliseconds
        """
        self.host_device = host_device
        self.receive_timeout_ms = receive_timeout_ms
        self.message_queue = queue.Queue()
        self.running = False
        self.listener_thread = None
        self.decoder = MessageDecoder()
        self._stats = {
            'messages_received': 0,
            'decode_successes': 0,
            'decode_failures': 0,
            'usb_errors': 0
        }
    
    def start_listening(self) -> bool:
        """
        Start the listener thread
        
        Returns:
            bool: True if started successfully, False if already running
        """
        if self.running:
            return False
        
        if not self.host_device.is_connected():
            raise USBCommunicationError("Host device must be connected before starting listener")
        
        self.running = True
        self.listener_thread = threading.Thread(target=self._listen_loop, daemon=True)
        self.listener_thread.start()
        return True
    
    def stop_listening(self) -> bool:
        """
        Stop the listener thread
        
        Returns:
            bool: True if stopped successfully
        """
        if not self.running:
            return False
        
        self.running = False
        if self.listener_thread:
            self.listener_thread.join(timeout=2.0)
            self.listener_thread = None
        return True
    
    def is_listening(self) -> bool:
        """Check if the listener is currently running"""
        return self.running and self.listener_thread is not None
    
    def _listen_loop(self):
        """Main listening loop (runs in separate thread)"""
        while self.running:
            try:
                # Try to receive raw data (with timeout to allow thread exit)
                raw_data = self.host_device.usb_device.receive_data(
                    timeout=self.receive_timeout_ms
                )
                
                if raw_data and len(raw_data) > 0:
                    self._stats['messages_received'] += 1
                    
                    # Try to decode the message
                    decoded_message = self.decoder.decode_message(raw_data)
                    
                    if decoded_message:
                        self._stats['decode_successes'] += 1
                        message_info = {
                            'timestamp': time.time(),
                            'message_type': self.decoder.get_message_type_name(decoded_message),
                            'message': decoded_message,
                            'raw_data': raw_data,
                            'decoded': True
                        }
                    else:
                        self._stats['decode_failures'] += 1
                        message_info = {
                            'timestamp': time.time(),
                            'message_type': 'Unknown',
                            'message': None,
                            'raw_data': raw_data,
                            'decoded': False
                        }
                    
                    self.message_queue.put(message_info)
                
            except USBCommunicationError as e:
                if "timeout" not in str(e).lower():
                    self._stats['usb_errors'] += 1
                # Continue listening even on timeouts and some errors
                time.sleep(0.01)
            except Exception as e:
                self._stats['usb_errors'] += 1
                # Log unexpected errors but continue
                time.sleep(0.1)
    
    def get_message(self, timeout: Optional[float] = None) -> Optional[dict]:
        """
        Get next message from queue (blocking)
        
        Args:
            timeout: Maximum time to wait for message (None = block indefinitely)
            
        Returns:
            Message info dict or None if timeout
        """
        try:
            return self.message_queue.get(timeout=timeout)
        except queue.Empty:
            return None
    
    def get_message_nowait(self) -> Optional[dict]:
        """
        Get next message from queue (non-blocking)
        
        Returns:
            Message info dict or None if no messages available
        """
        try:
            return self.message_queue.get_nowait()
        except queue.Empty:
            return None
    
    def has_messages(self) -> bool:
        """Check if there are pending messages in the queue"""
        return not self.message_queue.empty()
    
    def clear_messages(self) -> int:
        """
        Clear all pending messages from the queue
        
        Returns:
            int: Number of messages cleared
        """
        count = 0
        while not self.message_queue.empty():
            try:
                self.message_queue.get_nowait()
                count += 1
            except queue.Empty:
                break
        return count
    
    def get_stats(self) -> dict:
        """
        Get listener statistics
        
        Returns:
            dict: Statistics including message counts and error rates
        """
        stats = self._stats.copy()
        stats['queue_size'] = self.message_queue.qsize()
        stats['is_listening'] = self.is_listening()
        return stats
    
    def reset_stats(self):
        """Reset all statistics counters"""
        self._stats = {
            'messages_received': 0,
            'decode_successes': 0,
            'decode_failures': 0,
            'usb_errors': 0
        }


class USBMessageHandler:
    """
    Advanced message handler with callback support and filtering
    
    This class provides a framework for handling different types of USB messages
    with custom callbacks, filtering, and statistics tracking.
    """
    
    def __init__(self):
        """Initialize the message handler"""
        self.message_callbacks = {}
        self.message_filters = {}
        self.message_stats = {}
        self.global_callback = None
    
    def register_callback(self, message_type: type, callback) -> None:
        """
        Register a callback for specific message type
        
        Args:
            message_type: Type of message to handle (e.g., PluginData)
            callback: Function to call when message is received
                     Signature: callback(message, message_info)
        """
        self.message_callbacks[message_type] = callback
    
    def register_filter(self, message_type: type, filter_func) -> None:
        """
        Register a filter for specific message type
        
        Args:
            message_type: Type of message to filter
            filter_func: Function that returns True if message should be processed
                        Signature: filter_func(message, message_info) -> bool
        """
        self.message_filters[message_type] = filter_func
    
    def set_global_callback(self, callback) -> None:
        """
        Set a global callback that receives all messages
        
        Args:
            callback: Function to call for all messages
                     Signature: callback(message, message_info)
        """
        self.global_callback = callback
    
    def handle_message(self, message_info: dict) -> bool:
        """
        Handle incoming message with callbacks and filters
        
        Args:
            message_info: Message info dict from USBDataListener
            
        Returns:
            bool: True if message was processed, False if filtered out
        """
        if not message_info.get('decoded', False):
            # Handle unknown messages
            if self.global_callback:
                try:
                    self.global_callback(None, message_info)
                except Exception:
                    pass
            return False
        
        message = message_info['message']
        message_type = type(message)
        type_name = message_info['message_type']
        
        # Update statistics
        self.message_stats[type_name] = self.message_stats.get(type_name, 0) + 1
        
        # Apply filter if registered
        if message_type in self.message_filters:
            try:
                if not self.message_filters[message_type](message, message_info):
                    return False  # Message filtered out
            except Exception:
                return False  # Filter error, skip message
        
        # Call global callback first
        if self.global_callback:
            try:
                self.global_callback(message, message_info)
            except Exception:
                pass  # Don't let global callback errors stop specific handlers
        
        # Call specific callback if registered
        if message_type in self.message_callbacks:
            try:
                self.message_callbacks[message_type](message, message_info)
                return True
            except Exception:
                return False  # Callback error
        
        return True  # Message processed (even if no specific callback)
    
    def get_stats(self) -> dict:
        """Get message processing statistics"""
        return self.message_stats.copy()
    
    def reset_stats(self) -> None:
        """Reset all statistics"""
        self.message_stats.clear()