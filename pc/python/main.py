from enum import Enum
from collections import namedtuple
from plugin_host.generated_types import *

from plugin_host.comms import USBHostDevice, USBCommunicationError, uuid_str_to_bytes, serialize_command
from plugin_host.generated_types import BLEProperties, BluetoothAddressType
from time import sleep
def main():
    """Main example function"""
    print("=== USB Host Device Example ===\n")
    
    # Method 1: Manual connection management
    print("1. Manual Connection Management:")
    host = USBHostDevice()
    
    try:
        # Connect to device
        print("Connecting to USB device...")
        delay = 0.5
        if host.connect(sleep_time=1.0):
            print("✓ Connected successfully")
            
            # Configure a peripheral
            print("Configuring peripheral...")
            host.configure_peripheral(
                name="Example Peripheral",
                addr=[0xA1, 0xA2, 0xA3, 0xA4, 0xB1, 0xB2]
            )
            print("✓ Peripheral configured")
            
            sleep(delay)
            
            # Configure peripheral security
            print("Configuring peripheral security...")
            host.configure_peripheral_security(passkey=123456)
            print("✓ Security configured with passkey: 123456")
            
            sleep(delay)
            # Configure a service
            print("Configuring service...")
            host.configure_service(uuid="87654321-4321-4321-4321-cba987654321")
            print("✓ Service configured")

            sleep(delay)
            # Configure a characteristic with properties
            print("Configuring characteristic...")
            host.configure_characteristic(
                uuid="abcd1234-5678-90ab-cdef-123456789abc",
                service_uuid="87654321-4321-4321-4321-cba987654321",
                properties=[BLEProperties.READ, BLEProperties.WRITE, BLEProperties.NOTIFY]
            )
            print("✓ Characteristic configured")
            sleep(delay)
            
            # # Query service information (this would receive a response)
            # print("Querying service information...")
            # try:
            #     service_info = host.get_service_info("87654321-4321-4321-4321-cba987654321")
            #     print(f"✓ Service exists: {service_info.exists}")
            #     print(f"  Characteristics: {len(service_info.characteristic_uuids)}")
            # except USBCommunicationError as e:
            #     print(f"⚠ Service query failed (expected if no device): {e}")
            
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

if __name__ == "__main__":
    main()