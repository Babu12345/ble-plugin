#!/usr/bin/env python3
"""
Example demonstrating USB data listening and message handling

This example shows how to:
1. Listen for incoming USB data in a background thread
2. Automatically decode different message types
3. Handle messages with callbacks and filters
4. Track statistics and manage the listening process
"""

import time
from plugin_host.comms import (
    USBHostDevice,
    USBDataListener, 
    USBMessageHandler,
    USBCommunicationError
)
from plugin_host.generated_types import (
    PluginData,
    PluginServiceInfoResponse,
    PluginCharacteristicInfoResponse,
    PluginConfigurationError,
    PluginDataSendType,
    BLEProperties,
    BluetoothAddressType
)

def demonstrate_basic_listening():
    """Demonstrate basic USB data listening"""
    print("=== Basic USB Data Listening Example ===\n")
    
    try:
        with USBHostDevice() as host:
            print("✓ Connected to USB device")
            
            # Create and start listener
            listener = USBDataListener(host, receive_timeout_ms=1000)
            print("✓ Created USB data listener")
            
            listener.start_listening()
            print("✓ Started listening for incoming data")
            
            # Send some commands that might generate responses
            print("\n📤 Sending commands to potentially generate responses...")
            try:
                host.configure_peripheral("ListenerDemo", "demo-uuid-123")
                host.configure_service("service-uuid-456")
                
                # This should generate a response
                service_info = host.get_service_info("service-uuid-456")
                print(f"✓ Got service response: exists={service_info.exists}")
                
            except USBCommunicationError as e:
                print(f"⚠ Command failed (expected without physical device): {e}")
            
            # Listen for messages for a few seconds
            print(f"\n🎧 Listening for messages for 5 seconds...")
            start_time = time.time()
            message_count = 0
            
            while time.time() - start_time < 5.0:
                message_info = listener.get_message(timeout=0.5)
                
                if message_info:
                    message_count += 1
                    print(f"\n📨 Message #{message_count}:")
                    print(f"   Type: {message_info['message_type']}")
                    print(f"   Decoded: {message_info['decoded']}")
                    print(f"   Timestamp: {time.ctime(message_info['timestamp'])}")
                    
                    if message_info['decoded'] and message_info['message']:
                        message = message_info['message']
                        print(f"   Content: {message}")
                    else:
                        raw_data = message_info['raw_data']
                        preview = raw_data[:20].hex() if len(raw_data) > 20 else raw_data.hex()
                        print(f"   Raw data: {preview}{'...' if len(raw_data) > 20 else ''}")
                else:
                    print(".", end="", flush=True)
            
            # Stop listening and show statistics
            listener.stop_listening()
            print(f"\n\n✓ Stopped listening. Processed {message_count} messages.")
            
            stats = listener.get_stats()
            print(f"\n📊 Listener Statistics:")
            print(f"   Messages received: {stats['messages_received']}")
            print(f"   Decode successes: {stats['decode_successes']}")
            print(f"   Decode failures: {stats['decode_failures']}")
            print(f"   USB errors: {stats['usb_errors']}")
            print(f"   Final queue size: {stats['queue_size']}")
            
    except USBCommunicationError as e:
        print(f"⚠ USB Communication Error (expected without device): {e}")


def demonstrate_message_handling():
    """Demonstrate advanced message handling with callbacks"""
    print("\n" + "="*60 + "\n")
    print("=== Advanced Message Handling Example ===\n")
    
    # Set up message handler with callbacks
    handler = USBMessageHandler()
    
    # Statistics for demonstration
    callback_stats = {
        'plugin_data_count': 0,
        'service_responses': 0,
        'characteristic_responses': 0,
        'errors': 0,
        'unknown_messages': 0
    }
    
    def handle_plugin_data(data: PluginData, info: dict):
        """Handle incoming plugin data"""
        callback_stats['plugin_data_count'] += 1
        print(f"🔄 Plugin Data Received:")
        print(f"   Source: {data.src_id}")
        print(f"   Type: {data.send_type.name}")
        print(f"   Data: {data.data.hex() if len(data.data) <= 16 else data.data[:16].hex() + '...'}")
        
        # Handle different send types
        if data.send_type == PluginDataSendType.Notify:
            print(f"   📲 Processing notification from {data.src_id}")
        elif data.send_type == PluginDataSendType.Read:
            print(f"   📖 Processing read request from {data.src_id}")
        elif data.send_type == PluginDataSendType.Write:
            print(f"   ✏️  Processing write data from {data.src_id}")
    
    def handle_service_response(response: PluginServiceInfoResponse, info: dict):
        """Handle service information responses"""
        callback_stats['service_responses'] += 1
        print(f"🔍 Service Info Response:")
        print(f"   Service UUID: {response.service_uuid}")
        print(f"   Exists: {response.exists}")
        if response.exists:
            print(f"   Characteristics: {len(response.characteristic_uuids)}")
            for i, char_uuid in enumerate(response.characteristic_uuids[:3]):  # Show first 3
                print(f"     {i+1}. {char_uuid}")
            if len(response.characteristic_uuids) > 3:
                print(f"     ... and {len(response.characteristic_uuids) - 3} more")
    
    def handle_characteristic_response(response: PluginCharacteristicInfoResponse, info: dict):
        """Handle characteristic information responses"""
        callback_stats['characteristic_responses'] += 1
        print(f"🔧 Characteristic Info Response:")
        print(f"   Characteristic: {response.characteristic_uuid}")
        print(f"   Service: {response.service_uuid}")
        print(f"   Exists: {response.exists}")
        if response.exists:
            properties = [prop.name for prop in response.properties]
            print(f"   Properties: {', '.join(properties)}")
    
    def handle_error(error: PluginConfigurationError, info: dict):
        """Handle configuration errors"""
        callback_stats['errors'] += 1
        print(f"❌ Configuration Error: {error}")
    
    def global_message_handler(message, info: dict):
        """Global handler for all messages"""
        if message is None:
            callback_stats['unknown_messages'] += 1
            print(f"❓ Unknown message type received ({len(info['raw_data'])} bytes)")
        
        # Log all message timestamps for debugging
        print(f"   [Global] Message at {time.ctime(info['timestamp'])}")
    
    # Register all callbacks
    handler.register_callback(PluginData, handle_plugin_data)
    handler.register_callback(PluginServiceInfoResponse, handle_service_response)
    handler.register_callback(PluginCharacteristicInfoResponse, handle_characteristic_response)
    handler.register_callback(PluginConfigurationError, handle_error)
    handler.set_global_callback(global_message_handler)
    
    # Add a filter example - only process plugin data from specific sources
    def plugin_data_filter(data: PluginData, info: dict) -> bool:
        """Filter to only process data from allowed sources"""
        allowed_sources = ["demo-peripheral", "test-device", "example-sensor"]
        return data.src_id in allowed_sources or data.src_id.startswith("demo")
    
    handler.register_filter(PluginData, plugin_data_filter)
    
    print("✓ Message handler configured with callbacks and filters")
    print("✓ Registered handlers for:")
    print("   • PluginData (with source filtering)")
    print("   • PluginServiceInfoResponse")
    print("   • PluginCharacteristicInfoResponse") 
    print("   • PluginConfigurationError")
    print("   • Global handler for all messages")
    
    # Demonstrate with mock data (since we likely don't have a real device)
    print(f"\n🧪 Simulating message processing with mock data:")
    
    # Create some example messages to process
    example_messages = [
        {
            'timestamp': time.time(),
            'message_type': 'PluginData',
            'message': PluginData(
                src_id="demo-peripheral",
                send_type=PluginDataSendType.Notify,
                data=b"Hello from BLE device!"
            ),
            'raw_data': b'mock_data_1',
            'decoded': True
        },
        {
            'timestamp': time.time() + 1,
            'message_type': 'PluginServiceInfoResponse',
            'message': PluginServiceInfoResponse(
                service_uuid="battery-service-uuid",
                characteristic_uuids=["battery-level", "battery-status", "power-state"],
                exists=True
            ),
            'raw_data': b'mock_data_2',
            'decoded': True
        },
        {
            'timestamp': time.time() + 2,
            'message_type': 'PluginData',
            'message': PluginData(
                src_id="blocked-device",  # This should be filtered out
                send_type=PluginDataSendType.Write,
                data=b"This should be filtered"
            ),
            'raw_data': b'mock_data_3',
            'decoded': True
        },
        {
            'timestamp': time.time() + 3,
            'message_type': 'Unknown',
            'message': None,
            'raw_data': b'unknown_protocol_data',
            'decoded': False
        }
    ]
    
    # Process the mock messages
    for i, message_info in enumerate(example_messages, 1):
        print(f"\n--- Processing Mock Message {i} ---")
        processed = handler.handle_message(message_info)
        print(f"✓ Message processed: {processed}")
    
    # Show final statistics
    print(f"\n📊 Message Handler Statistics:")
    handler_stats = handler.get_stats()
    for msg_type, count in handler_stats.items():
        print(f"   {msg_type}: {count}")
    
    print(f"\n📈 Callback Statistics:")
    for stat_name, count in callback_stats.items():
        print(f"   {stat_name}: {count}")


def demonstrate_combined_listening_and_handling():
    """Demonstrate combining listener and handler for complete solution"""
    print("\n" + "="*60 + "\n")
    print("=== Combined Listening and Handling Example ===\n")
    
    try:
        with USBHostDevice() as host:
            print("✓ Connected to USB device")
            
            # Set up listener and handler
            listener = USBDataListener(host)
            handler = USBMessageHandler()
            
            # Set up a simple callback for demonstration
            def simple_message_processor(message, info):
                print(f"📦 Processed {info['message_type']} at {time.ctime(info['timestamp'])}")
            
            handler.set_global_callback(simple_message_processor)
            
            # Start listening
            listener.start_listening()
            print("✓ Started combined listening and handling")
            
            # Send some commands
            print(f"\n📤 Sending commands...")
            try:
                host.configure_peripheral("CombinedDemo", "combined-uuid")
                host.configure_service("combined-service")
                host.start_advertisement()
            except USBCommunicationError as e:
                print(f"⚠ Commands failed (expected): {e}")
            
            # Process messages for a few seconds
            print(f"\n🔄 Processing messages for 3 seconds...")
            start_time = time.time()
            processed_count = 0
            
            while time.time() - start_time < 3.0:
                message_info = listener.get_message(timeout=0.5)
                
                if message_info:
                    handler.handle_message(message_info)
                    processed_count += 1
                else:
                    print(".", end="", flush=True)
            
            # Clean up
            listener.stop_listening()
            
            print(f"\n\n✓ Processing complete")
            print(f"📊 Final Statistics:")
            
            listener_stats = listener.get_stats()
            handler_stats = handler.get_stats()
            
            print(f"   Listener - Messages received: {listener_stats['messages_received']}")
            print(f"   Listener - Decode successes: {listener_stats['decode_successes']}")
            print(f"   Handler - Messages processed: {processed_count}")
            
            for msg_type, count in handler_stats.items():
                print(f"   Handler - {msg_type}: {count}")
                
    except USBCommunicationError as e:
        print(f"⚠ USB Communication Error (expected without device): {e}")


def main():
    """Run all listening examples"""
    print("=== USB Data Listening and Message Handling Examples ===\n")
    print("These examples demonstrate how to listen for incoming USB data")
    print("and automatically process different message types with callbacks.\n")
    
    # Run all examples
    demonstrate_basic_listening()
    demonstrate_message_handling()
    demonstrate_combined_listening_and_handling()
    
    print("\n" + "="*60)
    print("=== All Listening Examples Complete ===")
    print("\nKey features demonstrated:")
    print("• Background thread listening for USB data")
    print("• Automatic message type detection and decoding")
    print("• Callback-based message processing")
    print("• Message filtering capabilities")
    print("• Statistics tracking for monitoring")
    print("• Graceful error handling and cleanup")
    print("\nNote: USB errors are expected when no physical device is connected.")
    print("="*60)


if __name__ == "__main__":
    main()