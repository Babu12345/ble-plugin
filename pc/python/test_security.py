#!/usr/bin/env python3
"""Test script for peripheral security configuration"""

from plugin_host.comms import USBHostDevice, USBCommunicationError

def test_security_configuration():
    """Test the configure_peripheral_security method"""
    print("=== Testing Peripheral Security Configuration ===\n")
    
    host = USBHostDevice()
    
    # Test valid passkeys
    valid_passkeys = [
        (0, "000000"),
        (123456, "123456"),
        (999999, "999999"),
        (1234, "001234"),
    ]
    
    print("Testing valid passkeys:")
    for passkey, display in valid_passkeys:
        try:
            # Note: This will fail if no USB device is connected
            # but we're testing the method interface
            host.configure_peripheral_security(passkey=passkey)
            print(f"✓ Passkey {display} accepted")
        except USBCommunicationError:
            print(f"✓ Passkey {display} accepted (USB device not connected)")
        except ValueError as e:
            print(f"✗ Unexpected error for passkey {display}: {e}")
    
    print("\nTesting invalid passkeys:")
    invalid_passkeys = [
        (-1, "negative number"),
        (1000000, "7 digits"),
        (9999999, "too large"),
    ]
    
    for passkey, description in invalid_passkeys:
        try:
            host.configure_peripheral_security(passkey=passkey)
            print(f"✗ Passkey {passkey} ({description}) should have been rejected")
        except ValueError as e:
            print(f"✓ Passkey {passkey} ({description}) correctly rejected: {e}")
        except USBCommunicationError:
            print(f"✗ Passkey {passkey} ({description}) should have raised ValueError, not USBCommunicationError")
    
    print("\n=== Test Complete ===")

if __name__ == "__main__":
    test_security_configuration()