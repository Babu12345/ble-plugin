#!/usr/bin/env python3
"""
Example usage of the new BLE plugin commands: clear_all_services and configure_profile

This script demonstrates how to use the new commands that were added to the BLE plugin:
1. HostCommandClearAllServices - Clear all configured services and characteristics
2. HostCommandConfigureProfile - Configure using predefined profiles

Author: Claude Code Assistant
Date: 2025-01-20
"""

import time
from plugin_host.comms import USBHostDevice
from plugin_host.generated_types import BLEProfile, BLEProperties

def demonstrate_new_commands():
    """Demonstrate the new clear_all_services and configure_profile commands"""
    
    print("🔌 BLE Plugin New Commands Demo")
    print("=" * 50)
    
    try:
        # Connect to the USB device
        with USBHostDevice() as device:
            print("✅ Connected to BLE plugin device")
            
            # Step 1: Configure a basic peripheral first
            print("\n📡 Step 1: Configuring peripheral...")
            device.configure_peripheral(
                name="DemoDevice", 
                addr=[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]
            )
            print("✅ Peripheral configured")
            
            # Step 2: Add some services and characteristics
            print("\n🔧 Step 2: Adding sample services...")
            device.configure_service("0x1800")  # Generic Access Service
            device.configure_characteristic(
                uuid="0x2A00",  # Device Name Characteristic
                service_uuid="0x1800",
                properties=[BLEProperties.READ]
            )
            
            device.configure_service("0x180F")  # Battery Service
            device.configure_characteristic(
                uuid="0x2A19",  # Battery Level Characteristic
                service_uuid="0x180F", 
                properties=[BLEProperties.READ, BLEProperties.NOTIFY]
            )
            print("✅ Sample services and characteristics configured")
            
            # Step 3: Verify services exist
            print("\n🔍 Step 3: Verifying services exist...")
            service_info = device.get_service_info("0x1800")
            print(f"📋 Generic Access Service exists: {service_info.exists}")
            print(f"📋 Characteristics: {[hex(uuid) for uuid in service_info.characteristic_uuids]}")
            
            service_info = device.get_service_info("0x180F")
            print(f"🔋 Battery Service exists: {service_info.exists}")
            print(f"🔋 Characteristics: {[hex(uuid) for uuid in service_info.characteristic_uuids]}")
            
            # Step 4: Demonstrate clear_all_services command
            print("\n🧹 Step 4: Clearing all services...")
            device.clear_all_services()
            print("✅ All services and characteristics cleared")
            
            # Step 5: Verify services are cleared
            print("\n🔍 Step 5: Verifying services are cleared...")
            service_info = device.get_service_info("0x1800")
            print(f"📋 Generic Access Service exists: {service_info.exists}")
            
            service_info = device.get_service_info("0x180F")
            print(f"🔋 Battery Service exists: {service_info.exists}")
            
            # Step 6: Add services again for profile demo
            print("\n🔧 Step 6: Adding services again for profile demo...")
            device.configure_service("0x1800")
            device.configure_characteristic(
                uuid="0x2A00",
                service_uuid="0x1800",
                properties=[BLEProperties.READ]
            )
            device.configure_service("0x180F")
            device.configure_characteristic(
                uuid="0x2A19",
                service_uuid="0x180F",
                properties=[BLEProperties.READ, BLEProperties.NOTIFY]
            )
            print("✅ Services reconfigured")
            
            # Step 7: Demonstrate configure_profile command
            print("\n📱 Step 7: Configuring custom profile...")
            device.configure_profile(BLEProfile.Custom)
            print("✅ Custom profile configured (server restarted with existing definitions)")
            
            # Step 8: Start advertising to complete the demo
            print("\n📡 Step 8: Starting advertisement...")
            device.start_advertisement(allow_multi_connect=False)
            print("✅ Advertisement started")
            
            print("\n🎉 Demo completed successfully!")
            print("💡 The device is now advertising and ready for BLE connections")
            print("🔄 Services have been configured using both manual setup and profile configuration")
            
    except Exception as e:
        print(f"❌ Error during demo: {e}")
        return False
    
    return True

def demonstrate_clear_services_only():
    """Demonstrate just the clear_all_services command"""
    
    print("\n🧹 Clear Services Only Demo")
    print("=" * 30)
    
    try:
        with USBHostDevice() as device:
            print("✅ Connected to BLE plugin device")
            
            # Configure minimal setup
            device.configure_peripheral(
                name="ClearDemo", 
                addr=[0x11, 0x22, 0x33, 0x44, 0x55, 0x66]
            )
            
            # Add a service
            device.configure_service("0x1801")  # Generic Attribute Service
            print("🔧 Added Generic Attribute Service")
            
            # Verify it exists
            service_info = device.get_service_info("0x1801")
            print(f"📋 Service exists before clear: {service_info.exists}")
            
            # Clear all services
            device.clear_all_services()
            print("🧹 Cleared all services")
            
            # Verify it's gone
            service_info = device.get_service_info("0x1801")
            print(f"📋 Service exists after clear: {service_info.exists}")
            
            print("✅ Clear services demo completed!")
            
    except Exception as e:
        print(f"❌ Error during clear demo: {e}")
        return False
    
    return True

if __name__ == "__main__":
    print("🚀 Starting BLE Plugin New Commands Demonstration")
    
    # Run the full demo
    if demonstrate_new_commands():
        print("\n" + "="*50)
        
        # Wait a bit and run the clear-only demo
        print("⏳ Waiting 2 seconds before next demo...")
        time.sleep(2)
        
        demonstrate_clear_services_only()
    
    print("\n🏁 All demonstrations completed!")