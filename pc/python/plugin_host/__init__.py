"""
Plugin Host Library

A Python library for communicating with BLE (Bluetooth Low Energy) plugin devices over USB.
This library provides high-level interfaces for configuring BLE peripherals, services, and 
characteristics, as well as handling real-time data communication with connected devices.

Key Features:
- USB communication with BLE plugin devices
- Protocol command serialization/deserialization using attrs2bin
- High-level device configuration methods
- Real-time message listening and handling
- Thread-safe data processing with callbacks and filters
- Comprehensive error handling and statistics

Main Classes:
- USBHostDevice: High-level interface for device communication
- USBDataListener: Thread-safe listener for incoming data
- USBMessageHandler: Advanced message handling with callbacks
- Protocol types: Various command and response classes

Example Usage:
    from plugin_host import USBHostDevice, BLEProperties, USBDataListener, USBMessageHandler
    
    # Basic configuration example
    with USBHostDevice() as device:
        device.configure_peripheral("MyDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
        device.configure_service("87654321-4321-4321-4321-cba987654321")
        device.configure_characteristic(
            "11111111-2222-3333-4444-555555555555",
            "87654321-4321-4321-4321-cba987654321",
            [BLEProperties.READ, BLEProperties.NOTIFY]
        )
        device.start_advertisement()
    
    # Advanced example with configuration and real-time listening
    def handle_plugin_data(message, message_info):
        print(f"Received data from {message.src_id}: {message.data}")
    
    def handle_service_info(message, message_info):
        print(f"Service {message.service_uuid} exists: {message.exists}")
    
    with USBHostDevice() as device:
        # Configure the BLE peripheral
        device.configure_peripheral("SensorDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
        device.configure_service("temperature-service-uuid")
        device.configure_characteristic(
            "temperature-char-uuid",
            "temperature-service-uuid",
            [BLEProperties.READ, BLEProperties.NOTIFY]
        )
        
        # Set up message handling
        handler = USBMessageHandler()
        handler.register_callback(PluginData, handle_plugin_data)
        handler.register_callback(PluginServiceInfoResponse, handle_service_info)
        
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
                    if message_info.get('decoded') and isinstance(message_info['message'], PluginData):
                        # Echo the data back as a notification
                        device.notify_characteristic_value(
                            address=b'\x01\x02\x03\x04\x05\x06',
                            address_type=BluetoothAddressType.Public,
                            characteristic_uuid="temperature-char-uuid",
                            service_uuid="temperature-service-uuid",
                            value=message_info['message'].data
                        )
        finally:
            listener.stop_listening()

Dependencies:
- attrs2bin: For protocol serialization/deserialization
- pyusb: For USB communication
- attr: For data class definitions

Protocol:
The library uses a custom USB protocol with length-prefixed messages:
[2-byte length][serialized data][padding to packet size]

All commands and responses are automatically serialized/deserialized using attrs2bin.
"""

from .generated_types import *
from .comms import *