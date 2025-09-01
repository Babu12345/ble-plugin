"""
Constants for the BLE plugin host communication system.

This file contains all the constants needed for USB communication protocol
and replaces the constants that were previously imported from generated_types.
"""

# ==================== USB PROTOCOL CONSTANTS ====================

# Magic number for message integrity validation (0xDEAD)
MESSAGE_MAGIC = 0xDEAD

# Default USB packet size for communication
DEFAULT_PACKET_SIZE = 64

# Message header field sizes in bytes
MESSAGE_MAGIC_BYTES = 2  # Size in bytes of the magic number field
MESSAGE_TYPE_ID_BYTES = 1  # Size in bytes of the message type identifier field  
DATA_BYTES_LENGTH_IN_BYTES = 2  # Size in bytes of the payload length field

# Total message header size
MESSAGE_HEADER_SIZE = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES  # 5 bytes

# ==================== USB DEVICE CONSTANTS ====================

# Default USB device IDs (MCP2221)
USB_VENDOR_ID = 0x04D8
USB_PRODUCT_ID = 0x00DD

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