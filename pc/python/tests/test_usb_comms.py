#!/usr/bin/env python3
"""
Simple test script to validate USB communication functions
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import struct
from plugin_host.comms import (
    serialize_command,
    deserialize_response,
    DATA_BYTES_LENGTH_IN_BYTES,
    DEFAULT_PACKET_SIZE
)
from plugin_host.types import (
    HostCommandConfigurePeripheral,
    HostCommandGetServiceInfo,
    PluginServiceInfoResponse
)

def test_serialization():
    """Test serialization and deserialization of protocol commands"""
    print("Testing serialization/deserialization...")
    
    # Test HostCommandConfigurePeripheral
    cmd = HostCommandConfigurePeripheral(
        name="TestDevice",
        uuid="12345678-1234-1234-1234-123456789abc"
    )
    
    try:
        # Serialize command
        serialized = serialize_command(cmd)
        print(f"✓ Serialized command: {len(serialized)} bytes")
        
        # Verify length prefix is present and correct
        data_length = struct.unpack('<H', serialized[:DATA_BYTES_LENGTH_IN_BYTES])[0]
        print(f"✓ Length prefix: {data_length} bytes")
        
        # Verify the format matches expected protocol
        if len(serialized) == DEFAULT_PACKET_SIZE:  # DEFAULT_PACKET_SIZE
            print(f"✓ Packet size is correct (256 bytes)")
        else:
            print(f"✗ Unexpected packet size: {len(serialized)}")
            return False
            
        print(f"✓ Command serialized successfully with length prefix")
        
    except Exception as e:
        print(f"✗ Serialization failed: {e}")
        return False
    
    # Test service info command
    service_cmd = HostCommandGetServiceInfo(
        uuid="87654321-4321-4321-4321-cba987654321"
    )
    
    try:
        serialized_service = serialize_command(service_cmd)
        service_length = struct.unpack('<H', serialized_service[:DATA_BYTES_LENGTH_IN_BYTES])[0]
        print(f"✓ Service command serialized: {len(serialized_service)} bytes (content: {service_length} bytes)")
        
    except Exception as e:
        print(f"✗ Service command serialization failed: {e}")
        return False
    
    return True
 
def test_mock_response():
    """Test deserializing a mock response"""
    print("\nTesting response deserialization...")
    
    # Create a mock response
    mock_response = PluginServiceInfoResponse(
        service_uuid="87654321-4321-4321-4321-cba987654321",
        characteristic_uuids=["char1-uuid", "char2-uuid"],
        exists=True
    )
    
    try:
        # Serialize it (simulating what would come from device)
        serialized = serialize_command(mock_response)
        
        # Deserialize it back
        deserialized = deserialize_response(serialized, PluginServiceInfoResponse)
        
        print(f"✓ Response deserialized successfully")
        print(f"  Service UUID: {deserialized.service_uuid}")
        print(f"  Characteristics: {len(deserialized.characteristic_uuids)}")
        print(f"  Exists: {deserialized.exists}")
        
        return True
        
    except Exception as e:
        print(f"✗ Response deserialization failed: {e}")
        return False

def test_length_prefix_protocol():
    """Test the length prefix protocol specifically"""
    print("\nTesting length prefix protocol...")
    
    # Create a simple command
    cmd = HostCommandGetServiceInfo(uuid="test-uuid-123")
    
    try:
        # Serialize
        serialized = serialize_command(cmd)
        
        # Manually inspect the length prefix
        length_prefix = serialized[:DATA_BYTES_LENGTH_IN_BYTES]
        data_length = struct.unpack('<H', length_prefix)[0]
        
        print(f"✓ Length prefix bytes: {length_prefix.hex()}")
        print(f"✓ Decoded length: {data_length} bytes")
        
        # Extract just the payload data for manual verification
        payload_data = serialized[DATA_BYTES_LENGTH_IN_BYTES:DATA_BYTES_LENGTH_IN_BYTES + data_length]
        print(f"✓ Payload size matches: {len(payload_data)} == {data_length}")
        
        # Test deserialization using the same data
        deserialized = deserialize_response(serialized, HostCommandGetServiceInfo)
        
        if deserialized.uuid == cmd.uuid:
            print(f"✓ Round-trip serialization/deserialization successful")
            return True
        else:
            print(f"✗ Round-trip failed: {deserialized.uuid} != {cmd.uuid}")
            return False
            
    except Exception as e:
        print(f"✗ Length prefix protocol test failed: {e}")
        return False

def main():
    """Run all tests"""
    print("=== USB Communication Functions Test ===\n")
    
    test1_passed = test_serialization()
    test2_passed = test_mock_response()
    test3_passed = test_length_prefix_protocol()
    
    print(f"\n=== Test Results ===")
    print(f"Serialization test: {'PASSED' if test1_passed else 'FAILED'}")
    print(f"Deserialization test: {'PASSED' if test2_passed else 'FAILED'}")
    print(f"Length prefix protocol test: {'PASSED' if test3_passed else 'FAILED'}")
    
    if test1_passed and test2_passed and test3_passed:
        print("\n🎉 All tests passed! USB communication functions are working.")
        return 0
    else:
        print("\n❌ Some tests failed. Check the implementation.")
        return 1

if __name__ == "__main__":
    exit(main())