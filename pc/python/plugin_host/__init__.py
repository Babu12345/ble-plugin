"""
Plugin Host Library

A Python library for communicating with BLE (Bluetooth Low Energy) plugin devices over USB.
This library provides high-level interfaces for configuring BLE peripherals, services, and 
characteristics, as well as handling real-time data communication with connected devices.

Key Features:
- USB communication with BLE plugin devices
- Protocol Buffers (protobuf) for command serialization/deserialization
- High-level device configuration methods
- Real-time message listening and handling
- Thread-safe data processing with callbacks and filters
- Comprehensive error handling and statistics

Main Classes:
- USBHostDevice: High-level interface for device communication
- USBDataListener: Thread-safe listener for incoming data
- USBMessageHandler: Advanced message handling with callbacks
- Protocol types: Available via protocol_pb2 module

Example Usage:
    from plugin_host import USBHostDevice, USBDataListener, USBMessageHandler
    import plugin_host.protocol_pb2 as protocol_pb2
    
    # Basic configuration example
    with USBHostDevice() as device:
        device.configure_peripheral("MyDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
        device.configure_service(0x8765)  # Use 16-bit hex UUID
        device.configure_characteristic(
            0x1111,  # Characteristic UUID as u16
            0x8765,  # Service UUID as u16
            [protocol_pb2.BleProperties.Read, protocol_pb2.BleProperties.Notify]
        )
        device.start_advertisement()
    
    # Advanced example with configuration and real-time listening
    def handle_plugin_data(message, message_info):
        print(f"Received data from {message.src_addr}: {message.data}")
    
    def handle_service_info(message, message_info):
        print(f"Service {message.service_uuid} exists: {message.exists}")
    
    with USBHostDevice() as device:
        # Configure the BLE peripheral
        device.configure_peripheral("SensorDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
        device.configure_service(0x1234)  # Use u16 UUID
        device.configure_characteristic(
            0x8765,  # Characteristic UUID as u16
            0x1234,  # Service UUID as u16
            [protocol_pb2.BleProperties.Read, protocol_pb2.BleProperties.Notify]
        )
        
        # Set up message handling
        handler = USBMessageHandler()
        handler.register_callback(protocol_pb2.PluginData, handle_plugin_data)
        handler.register_callback(protocol_pb2.PluginServiceInfoResponse, handle_service_info)
        
        # Start listening for incoming data
        listener = USBDataListener(device)
        listener.start_listening()
        
        # Start advertisement
        device.start_advertisement(allow_multi_connect=True)
        
        try:
            # Process incoming messages in real-time
            while True:
                message_info = listener.get_message(timeout=1.0)
                if message_info:
                    handler.handle_message(message_info)
                    
                    # Send notifications based on received data
                    if message_info.get('decoded') and isinstance(message_info['message'], protocol_pb2.PluginData):
                        # Echo the data back as a notification
                        device.notify_characteristic_value(
                            address=bytes([0x01, 0x02, 0x03, 0x04, 0x05, 0x06]),
                            address_type=protocol_pb2.BluetoothAddressType.Public,
                            characteristic_uuid=0x8765,  # Use u16 UUID
                            service_uuid=0x1234,  # Use u16 UUID
                            value=message_info['message'].data
                        )
        finally:
            listener.stop_listening()

Dependencies:
- protobuf: For protocol serialization/deserialization
- pyusb: For USB communication

Protocol:
The library uses Protocol Buffers with a custom USB message format:
[1-byte magic][2-byte type_id][2-byte length][protobuf data][padding]

All commands and responses are automatically serialized/deserialized using protobuf.
"""

# Import main classes for easier access
from .comms import (
    USBHostDevice, 
    USBCommunicationError, 
    USBDataListener, 
    USBMessageHandler,
    MessageDecoder,
    serialize_command,
    deserialize_response
)

# Re-export protocol types for convenience
from . import protocol_pb2

__all__ = [
    'USBHostDevice',
    'USBCommunicationError', 
    'USBDataListener',
    'USBMessageHandler',
    'MessageDecoder',
    'serialize_command',
    'deserialize_response',
    'protocol_pb2'
]