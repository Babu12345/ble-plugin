import usb.core
import usb.util
import struct
import threading
import time
import queue
import os
from typing import Any, Optional, Union
import plugin_host.protocol_pb2 as protocol_pb2

def parse_uuid_u16(uuid_value) -> int:
    """Parse UUID as u16 value
    
    Args:
        uuid_value: Either an integer or hex string (e.g., 0x1234, '0x1234', or 1234)
        
    Returns:
        u16 value for the UUID
        
    Raises:
        ValueError: If value is not valid or exceeds u16 maximum (65,535)
    """
    # Parse the value
    if isinstance(uuid_value, int):
        result = uuid_value
    elif isinstance(uuid_value, str):
        # Handle hex strings
        if uuid_value.startswith('0x') or uuid_value.startswith('0X'):
            result = int(uuid_value, 16)
        else:
            # Try to parse as decimal or hex without prefix
            try:
                result = int(uuid_value)
            except ValueError:
                result = int(uuid_value, 16)
    else:
        raise ValueError(f"Invalid UUID value type: {type(uuid_value)}")
    
    # Validate u16 range (0 to 65,535)
    if result < 0:
        raise ValueError(f"UUID value cannot be negative: {result}")
    if result > 0xFFFF:  # u16 max value
        raise ValueError(f"UUID value exceeds u16 maximum (65,535): {result}")
    
    return result

def validate_mac_address(addr: Union[bytes, bytearray, list]) -> Optional[str]:
    """Validate and convert MAC address to bytes
    
    Args:
        addr: MAC address as bytes, bytearray, or list of integers (0-255)
        
    Returns:
        None: If address is valid 
    """
    # Validate MAC address size
    if len(addr) != 6:
        return (
            "Invalid address.\n"
            f"MAC address must be exactly 6 bytes. Got {len(addr)} bytes.")
    
    # Validate random Bluetooth address patterns
    first_byte = addr[0]
    msb_bits = (first_byte >> 6) & 0x03  # Extract top 2 bits
    # Check for valid random address patterns
    if msb_bits == 0b10 or msb_bits == 0b01:  # Invalid patterns (MSB bits = 10 or 01)
        if msb_bits == 0b10:
            error_msg = "Invalid random address: MSB bits are 10 (binary)."
        else:
            error_msg = "Resolvable Private addresses not allowed for manual configuration."
        
        return (
            "Invalid MAC address.\n"
            f"{error_msg}\n\n"
            "Valid random address patterns for manual configuration:\n"
            "• Static Random: MSB bits = 11 (0xC0-0xFF)\n"
            "• Non-Resolvable Private: MSB bits = 00 (0x00-0x3F)"
        )
    
    # For Static Random and Non-Resolvable Private addresses,
    # check that remaining bits have at least one 0 and one 1
    if msb_bits == 0b11 or msb_bits == 0b00:  # Static Random or Non-Resolvable Private
        # Check all 46 bits (6 bytes minus 2 MSB bits)
        all_bits = 0
        for i, byte in enumerate(addr):
            if i == 0:
                # For first byte, only consider bottom 6 bits (exclude MSB bits)
                all_bits |= (byte & 0x3F) << (40)
            else:
                all_bits |= byte << ((5-i) * 8)
        
        # Check if all bits are 0 or all bits are 1 in the 46-bit range
        if all_bits == 0 or all_bits == 0x3FFFFFFFFFFF:  # All 46 bits same
            addr_type = "Static Random" if msb_bits == 0b11 else "Non-Resolvable Private"
            return ("Invalid Address\n" 
                f"Invalid {addr_type} address: remaining 46 bits must contain "
                "at least one 0 and at least one 1.")
    return None  # Address is valid

# Communicate between the host (PC) and the usb plugin


# Protocol/USB configuration constants
from plugin_host.constants import (
    MESSAGE_MAGIC, MESSAGE_MAGIC_BYTES, MESSAGE_TYPE_ID_BYTES, 
    DATA_BYTES_LENGTH_IN_BYTES, MESSAGE_HEADER_SIZE, 
    DEFAULT_PACKET_SIZE,USB_ENDPOINT_OUT,USB_ENDPOINT_IN,  USB_VENDOR_ID, USB_PRODUCT_ID, USB_TIMEOUT_MS,
    DEFAULT_COMMAND_DELAY, PROTOBUF_TO_TYPE_ID, TYPE_ID_TO_PROTOBUF
)

# Command delay configuration
_skip_command_delay = False
_custom_command_delay = None

def set_command_delay(delay_seconds: float) -> None:
    """
    Set custom command delay
    
    Args:
        delay_seconds: Delay in seconds (0.0 to disable delay)
    """
    global _custom_command_delay
    _custom_command_delay = delay_seconds

def get_command_delay() -> float:
    """
    Get the current command delay value
    
    Returns:
        float: Current delay in seconds
    """
    global _custom_command_delay
    return _custom_command_delay if _custom_command_delay is not None else DEFAULT_COMMAND_DELAY

def set_command_delay_enabled(enabled: bool) -> None:
    """
    Enable or disable command delay
    
    Args:
        enabled: If False, command delays will be skipped
    """
    global _skip_command_delay
    _skip_command_delay = not enabled

def is_command_delay_enabled() -> bool:
    """
    Check if command delay is enabled
    
    Returns:
        bool: True if delays are enabled, False if disabled
    """
    global _skip_command_delay
    # Check environment variable first, then global flag
    env_skip = os.environ.get('BLE_PLUGIN_SKIP_DELAY', '').lower() in ('true', '1', 'yes')
    return not (_skip_command_delay or env_skip)

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
        """Send raw bytes to the USB device with retry logic for I/O errors"""
        if not self.device:
            raise USBCommunicationError("Device not connected")
        
        max_retries = 3
        retry_count = 0
        
        while retry_count <= max_retries:
            try:
                return self.device.write(self.endpoint_out, data, timeout)
            except Exception as e:
                error_str = str(e).lower()
                
                # Check for device disconnection
                if any(pattern in error_str for pattern in ['errno 19', 'no such device', 'device disconnected']):
                    raise USBCommunicationError(f"Device disconnected: {e}")
                # Check for specific I/O error patterns
                elif any(pattern in error_str for pattern in ['errno 5', 'input/output error', 'i/o error']):
                    retry_count += 1
                    if retry_count <= max_retries:
                        # Exponential backoff for I/O errors
                        delay = min(0.1 * (2 ** (retry_count - 1)), 1.0)
                        time.sleep(delay)
                        continue
                    else:
                        raise USBCommunicationError(f"Failed to send data after {max_retries} retries due to I/O errors: {e}")
                elif 'busy' in error_str or 'resource' in error_str:
                    retry_count += 1
                    if retry_count <= max_retries:
                        # Shorter delay for busy errors
                        time.sleep(0.01)
                        continue
                    else:
                        raise USBCommunicationError(f"Failed to send data - device busy after {max_retries} retries: {e}")
                else:
                    # Other errors are not retried
                    raise USBCommunicationError(f"Failed to send data: {e}")
        
        # This should not be reached, but just in case
        raise USBCommunicationError("Unexpected error in send_data retry logic")
    
    def receive_data(self, size: int = DEFAULT_PACKET_SIZE, timeout: int = USB_TIMEOUT_MS) -> bytes:
        """Receive raw bytes from the USB device with retry logic for I/O errors"""
        if not self.device:
            raise USBCommunicationError("Device not connected")
        
        max_retries = 3
        retry_count = 0
        
        while retry_count <= max_retries:
            try:
                data = self.device.read(self.endpoint_in, size, timeout)
                return bytes(data)
            except Exception as e:
                error_str = str(e).lower()
                
                # Check for specific I/O error patterns
                if any(pattern in error_str for pattern in ['errno 5', 'input/output error', 'i/o error']):
                    retry_count += 1
                    if retry_count <= max_retries:
                        # Exponential backoff for I/O errors
                        delay = min(0.1 * (2 ** (retry_count - 1)), 1.0)
                        time.sleep(delay)
                        continue
                    else:
                        raise USBCommunicationError(f"Failed to receive data after {max_retries} retries due to I/O errors: {e}")
                elif 'timeout' in error_str:
                    # Timeout errors are passed through immediately (not retried)
                    raise USBCommunicationError(f"Failed to receive data: {e}")
                elif 'busy' in error_str or 'resource' in error_str:
                    retry_count += 1
                    if retry_count <= max_retries:
                        # Shorter delay for busy errors
                        time.sleep(0.01)
                        continue
                    else:
                        raise USBCommunicationError(f"Failed to receive data - device busy after {max_retries} retries: {e}")
                else:
                    # Other errors are not retried
                    raise USBCommunicationError(f"Failed to receive data: {e}")
        
        # This should not be reached, but just in case
        raise USBCommunicationError("Unexpected error in receive_data retry logic")


def serialize_command(command: Any) -> bytes:
    """
    Serialize a protocol command object to bytes using protobuf with full message header
    
    Format: [1-byte magic][2-byte type_id][2-byte length][serialized data][padding to DEFAULT_PACKET_SIZE]
    
    Args:
        command: Protobuf command object
        
    Returns:
        bytes: Serialized command data with full message header and padding
        
    Raises:
        USBCommunicationError: If serialization fails or protobuf not available
    """
    
    try:
        # Get message type ID for this command
        command_type = type(command)
        if command_type not in PROTOBUF_TO_TYPE_ID:
            raise USBCommunicationError(f"Unknown protobuf message type: {command_type}")
        
        message_type_id = PROTOBUF_TO_TYPE_ID[command_type]
        
        # Use protobuf to serialize the command
        serialized_data = command.SerializeToString()
        data_length = len(serialized_data)
        
        # Ensure the data fits within the packet size (accounting for full header)
        if data_length + MESSAGE_HEADER_SIZE > DEFAULT_PACKET_SIZE:
            raise USBCommunicationError(f"Command size ({data_length}) + header ({MESSAGE_HEADER_SIZE}) exceeds packet size ({DEFAULT_PACKET_SIZE})")
        
        # Create message header
        header = bytearray()
        
        # Add magic byte (0xDE)
        header.append(MESSAGE_MAGIC)
        
        # Add message type ID (2-byte little-endian)
        header.extend(struct.pack('<H', int(message_type_id)))
        
        # Add length (2-byte little-endian)
        header.extend(struct.pack('<H', data_length))
        
        # Combine header with serialized data
        complete_data = bytes(header) + serialized_data
        
        # Pad to packet size for consistent USB transfers
        padded_data = complete_data + b'\x00' * (DEFAULT_PACKET_SIZE - len(complete_data))
        return padded_data
        
    except Exception as e:
        raise USBCommunicationError(f"Failed to serialize protobuf command: {e}")


def deserialize_response(data: bytes, response_type: type = None) -> Any:
    """
    Deserialize bytes to a protocol response object using protobuf with full message header
    
    Format: [1-byte magic][2-byte type_id][2-byte length][serialized data][padding]
    
    Args:
        data: Raw bytes received from USB
        response_type: Optional expected protobuf response type class (if None, auto-detect from type ID)
        
    Returns:
        Any: Deserialized protobuf response object
        
    Raises:
        USBCommunicationError: If deserialization fails or protobuf not available
    """
    
    try:
        # Check minimum header size
        if len(data) < MESSAGE_HEADER_SIZE:
            raise USBCommunicationError(f"Data too short to contain message header: {len(data)} bytes")
        
        # Verify magic number
        magic = data[0]
        if magic != MESSAGE_MAGIC:
            raise USBCommunicationError(f"Invalid magic number: expected 0x{MESSAGE_MAGIC:02X}, got 0x{magic:02X}")
        
        # Extract message type ID (2-byte little-endian)
        type_id_bytes = data[MESSAGE_MAGIC_BYTES:MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES]
        type_id_byte = struct.unpack('<H', type_id_bytes)[0]
        
        # Extract data length
        length_start = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES
        length_end = length_start + DATA_BYTES_LENGTH_IN_BYTES
        data_length = struct.unpack('<H', data[length_start:length_end])[0]
        
        # Validate length
        if data_length > DEFAULT_PACKET_SIZE:
            raise USBCommunicationError(f"Data length ({data_length}) exceeds packet size ({DEFAULT_PACKET_SIZE})")
        
        # Determine response type from message ID if not provided
        if response_type is None:
            matching_protobuf_type = None
            for protobuf_type_id, protobuf_class in TYPE_ID_TO_PROTOBUF.items():
                if type_id_byte == protobuf_type_id:
                    matching_protobuf_type = protobuf_class
                    break
            
            if matching_protobuf_type is None:
                raise USBCommunicationError(f"No protobuf handler for message type ID: 0x{type_id_byte:02X}")
            response_type = matching_protobuf_type
        
        # Extract the actual serialized data using the length
        data_start = MESSAGE_HEADER_SIZE
        data_end = data_start + data_length
        
        if data_end > len(data):
            raise USBCommunicationError(f"Insufficient data: expected {data_length} bytes, got {len(data) - MESSAGE_HEADER_SIZE}")
        
        serialized_data = data[data_start:data_end]
        
        # Use protobuf to deserialize the response
        response = response_type()
        response.ParseFromString(serialized_data)
        return response
        
    except Exception as e:
        raise USBCommunicationError(f"Failed to deserialize protobuf response: {e}")

def usb_send_command(device: USBDevice, command: Any) -> bool:
    """
    Send a protocol command over USB with optional delay after sending
    
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
        result = bytes_sent == len(serialized_data)
        
        # Add delay after sending if enabled
        if is_command_delay_enabled():
            delay = get_command_delay()
            if delay > 0:
                time.sleep(delay)
        
        return result
    except Exception as e:
        raise USBCommunicationError(f"Failed to send command: {e}")

def usb_receive_response(device: USBDevice, response_type: type = None) -> Any:
    """
    Receive and deserialize a protocol response over USB
    
    Args:
        device: Connected USB device
        response_type: Optional expected response type class (auto-detected if None)
        
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

def usb_send_and_receive(device: USBDevice, command: Any, response_type: type = None) -> Any:
    """
    Send a command and receive response in one operation
    
    Args:
        device: Connected USB device
        command: Protocol command object to send
        response_type: Optional expected response type class (auto-detected if None)
        
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
    
    def __init__(self, vendor_id: int = USB_VENDOR_ID, product_id: int = USB_PRODUCT_ID, default_command_delay: float = DEFAULT_COMMAND_DELAY):
        """
        Initialize the USB Host Device
        
        Args:
            vendor_id: USB vendor ID of the plugin device
            product_id: USB product ID of the plugin device
            default_command_delay: Default delay in seconds between commands (default: DEFAULT_COMMAND_DELAY)
        """
        self.usb_device = USBDevice(vendor_id, product_id)
        self._connected = False
        set_command_delay(default_command_delay)  # Set default command delay
    
    def connect(self, sleep_time: float = 0.0) -> bool:
        """
        Connect to the USB plugin device
        
        Returns:
            bool: True if connection successful
            
        Raises:
            USBCommunicationError: If connection fails
        """
        result = self.usb_device.connect()
        self._connected = result
        time.sleep(sleep_time)
        return result
    
    def disconnect(self) -> None:
        """Disconnect from the USB plugin device"""
        self.usb_device.disconnect()
        self._connected = False
    
    def is_connected(self) -> bool:
        """Check if device is connected"""
        return self._connected
    
    # Host Command Methods
    
    def configure_peripheral(self, name: str, addr: bytes) -> None:
        """
        Configure a peripheral device
        
        Args:
            name: Peripheral name (max 32 characters)
            addr: Peripheral MAC address as list of 6 bytes
            
        Raises:
            USBCommunicationError: If sending fails
        """
        validation = validate_mac_address(addr)
        if validation is not None:
            raise ValueError(validation)
        cmd = protocol_pb2.HostCommandConfigurePeripheral(name=name, addr=bytes(addr))
        usb_send_command(self.usb_device, cmd)
    
    def configure_service(self, uuid: str) -> None:
        """
        Configure a service
        
        Args:
            uuid: Service UUID as string
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = protocol_pb2.HostCommandConfigureService(uuid=parse_uuid_u16(uuid))
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
        cmd = protocol_pb2.HostCommandConfigureCharacteristic(
            uuid=parse_uuid_u16(uuid),
            service_uuid=parse_uuid_u16(service_uuid),
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
        cmd = protocol_pb2.HostCommandConfigureCharacteristicRead(
            uuid=parse_uuid_u16(uuid),
            service_uuid=parse_uuid_u16(service_uuid),
            value=value
        )
        usb_send_command(self.usb_device, cmd)
    
    def get_service_info(self, uuid: str) -> (protocol_pb2.PluginServiceInfoResponse):
        """
        Get service information
        
        Args:
            uuid: Service UUID as string
            
        Returns:
            PluginServiceInfoResponse: Service information response
            
        Raises:
            USBCommunicationError: If communication fails
        """
        cmd = protocol_pb2.HostCommandGetServiceInfo(uuid=parse_uuid_u16(uuid))
        return usb_send_and_receive(self.usb_device, cmd, protocol_pb2.PluginServiceInfoResponse)
    
    def get_characteristic_info(self, characteristic_uuid: str, service_uuid: str) -> protocol_pb2.PluginCharacteristicInfoResponse:
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
        cmd = protocol_pb2.HostCommandGetCharacteristicInfo(
            characteristic_uuid=parse_uuid_u16(characteristic_uuid),
            service_uuid=parse_uuid_u16(service_uuid)
        )
        return usb_send_and_receive(self.usb_device, cmd, protocol_pb2.PluginCharacteristicInfoResponse)
    
    def configure_peripheral_security(self, passkey: int) -> None:
        """
        Configure peripheral security settings
        
        Args:
            passkey: 6-digit numeric passkey for pairing (e.g., 123456)
            
        Raises:
            USBCommunicationError: If sending fails
            ValueError: If passkey is not a valid 6-digit number
        """
        if not (0 <= passkey <= 999999):
            raise ValueError("Passkey must be a 6-digit number between 000000 and 999999")
        
        cmd = protocol_pb2.HostCommandConfigurePeripheralSecurity(passkey=passkey)
        usb_send_command(self.usb_device, cmd)
    
    def start_advertisement(self, allow_multi_connect: bool = False) -> None:
        """
        Start advertisement
        
        Note: On the first call, this will auto-configure using any predefined profile settings.
        Subsequent calls require explicit configuration via configure_profile() or manual service setup.
        
        Args:
            allow_multi_connect: Allow multiple central connections
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = protocol_pb2.HostCommandStartAdvertisement(allow_multi_connect=allow_multi_connect)
        usb_send_command(self.usb_device, cmd)
    
    def stop_advertisement(self) -> None:
        """
        Stop BLE advertisement
        
        Stops all ongoing BLE advertisements.
        
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = protocol_pb2.HostCommandStopAdvertisement()
        usb_send_command(self.usb_device, cmd)
    
    def notify_characteristic_value(self, address: bytes, address_type: protocol_pb2.BluetoothAddressType, 
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
        cmd = protocol_pb2.HostCommandNotifyCharacteristicValue(
            address=address,
            address_type=address_type,
            characteristic_uuid=parse_uuid_u16(characteristic_uuid),
            service_uuid=parse_uuid_u16(service_uuid),
            value=value
        )
        usb_send_command(self.usb_device, cmd)
    
    def configure_profile(self, profile: protocol_pb2.BleProfile, delay = 0.05) -> None:
        """
        Configure BLE profile using predefined settings
        
        This command configures the BLE device using a predefined profile,
        which applies all previously configured services and characteristics.
        
        Args:
            profile: BLE profile type to configure (currently only BleProfile.Custom supported)
            delay: Optional delay in seconds after sending the command (default: 0.05). Needed as it might be required
            to wait for BLE central devices to disconnect and advertising to stop before reconfiguring.
            
        Raises:
            USBCommunicationError: If sending fails
        """
        cmd = protocol_pb2.HostCommandConfigureProfile(profile=profile)
        usb_send_command(self.usb_device, cmd)
        time.sleep(delay)
    
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
    
    def receive_response(self, response_type: type = None) -> Any:
        """
        Receive and deserialize a protocol response
        
        Args:
            response_type: Optional expected response type class (auto-detected if None)
            
        Returns:
            Any: Deserialized response object
            
        Raises:
            USBCommunicationError: If receiving fails
        """
        return usb_receive_response(self.usb_device, response_type)
    
    def send_and_receive(self, command: Any, response_type: type = None) -> Any:
        """
        Send a command and receive response with automatic serialization/deserialization
        
        Args:
            command: Protocol command object to send
            response_type: Optional expected response type class (auto-detected if None)
            
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
        # Explicitly mark parameters as unused to avoid linter warnings
        _ = exc_type, exc_val, exc_tb
        self.disconnect()
        return False  # Don't suppress exceptions


class MessageDecoder:
    """Utility class to decode incoming plugin messages using message type IDs"""
    
    @classmethod
    def decode_message(cls, raw_data: bytes) -> Optional[Any]:
        """
        Decode raw bytes using protobuf deserialization
        
        Args:
            raw_data: Raw bytes received from USB device
            
        Returns:
            Decoded protobuf message object or None if decoding failed
        """
        try:
            decoded = deserialize_response(raw_data)
            return decoded
        except Exception:
            # Return None if decoding failed
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
        self._stop_event = threading.Event()
        self.listener_thread = None
        self.decoder = MessageDecoder()
        self._stats_lock = threading.Lock()
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
        if self.listener_thread and self.listener_thread.is_alive():
            return False
        
        if not self.host_device.is_connected():
            raise USBCommunicationError("Host device must be connected before starting listener")
        
        # Clear stop event and reset stats
        self._stop_event.clear()
        with self._stats_lock:
            self._stats = {
                'messages_received': 0,
                'decode_successes': 0,
                'decode_failures': 0,
                'usb_errors': 0
            }
        
        self.listener_thread = threading.Thread(target=self._listen_loop, daemon=True)
        self.listener_thread.start()
        return True
    
    def stop_listening(self) -> bool:
        """
        Stop the listener thread
        
        Returns:
            bool: True if stopped successfully
        """
        if not self.listener_thread or not self.listener_thread.is_alive():
            return False
        
        # Signal the thread to stop
        self._stop_event.set()
        
        # Wait for the thread to finish
        if self.listener_thread:
            self.listener_thread.join(timeout=2.0)
            # If thread didn't stop gracefully, it will be cleaned up when the object is destroyed
            self.listener_thread = None
        return True
    
    def is_listening(self) -> bool:
        """Check if the listener is currently running"""
        return self.listener_thread is not None and self.listener_thread.is_alive() and not self._stop_event.is_set()
    
    def _listen_loop(self):
        """Main listening loop (runs in separate thread)"""
        while not self._stop_event.is_set():
            try:
                # Try to receive raw data (with timeout to allow thread exit)
                raw_data = self.host_device.usb_device.receive_data(
                    timeout=self.receive_timeout_ms
                )
                
                if raw_data and len(raw_data) > 0:
                    with self._stats_lock:
                        self._stats['messages_received'] += 1
                    
                    # Try to decode the message
                    decoded_message = self.decoder.decode_message(raw_data)
                    
                    if decoded_message:
                        with self._stats_lock:
                            self._stats['decode_successes'] += 1
                        message_info = {
                            'timestamp': time.time(),
                            'message_type': self.decoder.get_message_type_name(decoded_message),
                            'message': decoded_message,
                            'raw_data': raw_data,
                            'decoded': True
                        }
                    else:
                        with self._stats_lock:
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
                    with self._stats_lock:
                        self._stats['usb_errors'] += 1
                # Continue listening even on timeouts and some errors
                # Check stop event more frequently during error recovery
                if not self._stop_event.wait(0.01):
                    continue
                else:
                    break
            except Exception as e:
                with self._stats_lock:
                    self._stats['usb_errors'] += 1
                # Log unexpected errors but continue
                # Check stop event more frequently during error recovery
                if not self._stop_event.wait(0.1):
                    continue
                else:
                    break
    
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
        with self._stats_lock:
            stats = self._stats.copy()
        stats['queue_size'] = self.message_queue.qsize()
        stats['is_listening'] = self.is_listening()
        return stats
    
    def reset_stats(self):
        """Reset all statistics counters"""
        with self._stats_lock:
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