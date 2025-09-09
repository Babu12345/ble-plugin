"""
Constants for the BLE plugin host communication system.

This file contains all the constants needed for USB communication protocol
and replaces the constants that were previously imported from generated_types.
"""

# ==================== USB PROTOCOL CONSTANTS ====================

# Magic number for message integrity validation (0xDE)
MESSAGE_MAGIC = 0xDE

# Default USB packet size for communication
DEFAULT_PACKET_SIZE = 64

# Message header field sizes in bytes
MESSAGE_MAGIC_BYTES = 1  # Size in bytes of the magic number field
MESSAGE_TYPE_ID_BYTES = 2  # Size in bytes of the message type identifier field  
DATA_BYTES_LENGTH_IN_BYTES = 2  # Size in bytes of the payload length field

# Total message header size
MESSAGE_HEADER_SIZE = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES  # 5 bytes

# ==================== USB DEVICE CONSTANTS ====================

# Default USB device IDs (MCP2221)
USB_VENDOR_ID = 0xffff
USB_PRODUCT_ID = 0xffff

# USB endpoint addresses
USB_ENDPOINT_OUT = 0x02
USB_ENDPOINT_IN = 0x81

# USB communication timeout in milliseconds
USB_TIMEOUT_MS = 1000

# ==================== BLE CONSTANTS ====================

# Maximum size for BLE peripheral device names
MAX_NAME_SIZE = 30

# Maximum size for characteristic properties
MAX_PROPERTIES = 4

# Maximum characteristics per service
MAX_CHARACTERISTICS_PER_SERVICE = 16

# ==================== TIMING CONSTANTS ====================

# Default command delay in seconds
DEFAULT_COMMAND_DELAY = 0.0

# ==================== MESSAGE TYPE MAPPINGS ====================

# Import protocol definitions for message type mappings
import plugin_host.protocol_pb2 as protocol_pb2

# Master mapping from protobuf classes to their message type IDs
PROTOBUF_TO_TYPE_ID = {
    # Commands (sent TO the device)
    protocol_pb2.HostCommandConfigurePeripheral: protocol_pb2.MessageTypeId.TypeHostCommandConfigurePeripheral,
    protocol_pb2.HostCommandConfigurePeripheralSecurity: protocol_pb2.MessageTypeId.TypeHostCommandConfigurePeripheralSecurity,
    protocol_pb2.HostCommandConfigureService: protocol_pb2.MessageTypeId.TypeHostCommandConfigureService,
    protocol_pb2.HostCommandConfigureCharacteristic: protocol_pb2.MessageTypeId.TypeHostCommandConfigureCharacteristic,
    protocol_pb2.HostCommandConfigureCharacteristicRead: protocol_pb2.MessageTypeId.TypeHostCommandConfigureCharacteristicRead,
    protocol_pb2.HostCommandGetServiceInfo: protocol_pb2.MessageTypeId.TypeHostCommandGetServiceInfo,
    protocol_pb2.HostCommandGetCharacteristicInfo: protocol_pb2.MessageTypeId.TypeHostCommandGetCharacteristicInfo,
    protocol_pb2.HostCommandStartAdvertisement: protocol_pb2.MessageTypeId.TypeHostCommandStartAdvertisement,
    protocol_pb2.HostCommandStopAdvertisement: protocol_pb2.MessageTypeId.TypeHostCommandStopAdvertisement,
    protocol_pb2.HostCommandNotifyCharacteristicValue: protocol_pb2.MessageTypeId.TypeHostCommandNotifyCharacteristicValue,
    protocol_pb2.HostCommandConfigureProfile: protocol_pb2.MessageTypeId.TypeHostCommandConfigureProfile,
    # Responses (sent FROM the device, included for testing purposes)
    protocol_pb2.PluginData: protocol_pb2.MessageTypeId.TypePluginData,
    protocol_pb2.PluginConfigurationError: protocol_pb2.MessageTypeId.TypePluginConfigurationError,
    protocol_pb2.PluginServiceInfoResponse: protocol_pb2.MessageTypeId.TypePluginServiceInfoResponse,
    protocol_pb2.PluginCharacteristicInfoResponse: protocol_pb2.MessageTypeId.TypePluginCharacteristicInfoResponse,
    protocol_pb2.PluginAuthenticationCompletedResponse: protocol_pb2.MessageTypeId.TypePluginAuthenticationCompletedResponse,
}

# Generate reverse mapping using dictionary comprehension
TYPE_ID_TO_PROTOBUF = {type_id: protobuf_class for protobuf_class, type_id in PROTOBUF_TO_TYPE_ID.items()}