#!/usr/bin/env python3
"""
Example usage of the USBHostDevice class for BLE plugin communication

This example demonstrates how to use the USBHostDevice class to:
1. Connect to a USB BLE plugin device
2. Configure peripherals, services, and characteristics  
3. Query service and characteristic information
4. Handle automatic serialization/deserialization
"""

from plugin_host.comms import USBHostDevice, USBCommunicationError
from plugin_host.generated_types import BLEProperties, BluetoothAddressType

def main():
    """Main example function"""
    print("=== USB Host Device Example ===\n")
    
    # Method 1: Manual connection management
    print("1. Manual Connection Management:")
    host = USBHostDevice()
    
    try:
        # Connect to device
        print("Connecting to USB device...")
        if host.connect():
            print("✓ Connected successfully")
            
            # Configure a peripheral
            print("Configuring peripheral...")
            host.configure_peripheral(
                name="ExampleDevice", 
                uuid="12345678-1234-1234-1234-123456789abc"
            )
            print("✓ Peripheral configured")
            
            # Configure a service
            print("Configuring service...")
            host.configure_service(uuid="87654321-4321-4321-4321-cba987654321")
            print("✓ Service configured")
            
            # Configure a characteristic with properties
            print("Configuring characteristic...")
            host.configure_characteristic(
                uuid="abcd1234-5678-90ab-cdef-123456789abc",
                service_uuid="87654321-4321-4321-4321-cba987654321",
                properties=[BLEProperties.READ, BLEProperties.WRITE, BLEProperties.NOTIFY]
            )
            print("✓ Characteristic configured")
            
            # Query service information (this would receive a response)
            print("Querying service information...")
            try:
                service_info = host.get_service_info("87654321-4321-4321-4321-cba987654321")
                print(f"✓ Service exists: {service_info.exists}")
                print(f"  Characteristics: {len(service_info.characteristic_uuids)}")
            except USBCommunicationError as e:
                print(f"⚠ Service query failed (expected if no device): {e}")
            
            # Start advertisement
            print("Starting advertisement...")
            host.start_advertisement(allow_multi_connect=True)
            print("✓ Advertisement started")
            
        else:
            print("✗ Failed to connect")
            
    except USBCommunicationError as e:
        print(f"⚠ USB Communication Error (expected if no device): {e}")
    finally:
        # Always disconnect
        host.disconnect()
        print("✓ Disconnected")
    
    print("\n" + "="*50 + "\n")
    
    # Method 2: Context manager (automatic connection/disconnection)
    print("2. Context Manager Usage:")
    
    try:
        with USBHostDevice() as host_device:
            print("✓ Connected via context manager")
            
            # Configure characteristic read value
            host_device.configure_characteristic_read(
                uuid="char-read-uuid",
                service_uuid="service-uuid", 
                value=b"default_value"
            )
            print("✓ Characteristic read configured")
            
            # Notify a characteristic value
            host_device.notify_characteristic_value(
                address=b'\x12\x34\x56\x78\x9a\xbc',  # Example MAC address
                address_type=BluetoothAddressType.Public,
                characteristic_uuid="char-notify-uuid",
                service_uuid="service-uuid",
                value=b"notification_data"
            )
            print("✓ Characteristic notification sent")
            
    except USBCommunicationError as e:
        print(f"⚠ USB Communication Error (expected if no device): {e}")
    
    print("✓ Auto-disconnected via context manager")
    
    print("\n" + "="*50 + "\n")
    
    # Method 3: Generic command sending
    print("3. Generic Command Usage:")
    
    from plugin_host.generated_types import HostCommandGetServiceInfo
    
    host = USBHostDevice()
    try:
        if host.connect():
            print("✓ Connected for generic command test")
            
            # Create a command manually
            custom_command = HostCommandGetServiceInfo(uuid="custom-service-uuid")
            
            # Send using generic method
            host.send_command(custom_command)
            print("✓ Generic command sent")
            
            # You could also receive a response generically:
            # response = host.receive_response(PluginServiceInfoResponse)
            
    except USBCommunicationError as e:
        print(f"⚠ USB Communication Error (expected if no device): {e}")
    finally:
        host.disconnect()
        print("✓ Disconnected")
    
    print("\n=== Example Complete ===")
    print("Note: USB communication errors are expected when no physical device is connected.")
    print("The example demonstrates the API usage patterns.")

if __name__ == "__main__":
    main()