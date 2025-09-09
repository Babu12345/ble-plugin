
## Create a virtual environment
python3 -m venv /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python

## Activate the virtual environment
source /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python/bin/activate

# Installations
pip install pyusb
pip install libusb
pip install pytest
pip install protobuf

brew install libusb # Make sure that this is run
brew install python-tk # for the gui


then add these lines to pytest.ini
```
[pytest]
pythonpath = .
```

# Running tests
pytest <DIRECTORY_OF_TESTS>

# BLE Plugin Python Library

This Python library provides a high-level interface for communicating with BLE plugin devices over USB using Protocol Buffers (protobuf) for serialization.

## Architecture

The library uses **Protocol Buffers (protobuf)** exclusively for message serialization between the host PC and the USB plugin device. All communication follows this protocol:

- **Message Format**: `[1-byte magic][2-byte type_id][2-byte length][protobuf data][padding]`
- **Magic Number**: `0xDE` for message integrity validation
- **Packet Size**: Fixed 64-byte USB packets
- **Constants**: Centralized in `plugin_host.constants` module

### Key Modules

- `plugin_host.comms` - Core USB communication and device management
- `plugin_host.protocol_pb2` - Generated protobuf message types
- `plugin_host.constants` - Protocol constants and USB configuration

### Type System

All message types are defined in protobuf and available via:
```python
import plugin_host.protocol_pb2 as protocol_pb2

# Enums
protocol_pb2.BleProperties.Read
protocol_pb2.PluginDataSendType.NotifyType
protocol_pb2.BluetoothAddressType.Public

# Messages
protocol_pb2.HostCommandConfigurePeripheral(...)
protocol_pb2.PluginData(...)
```

## Available Commands

### Core Commands

- `configure_peripheral(name, addr)` - Configure BLE peripheral with name and address
- `configure_service(uuid)` - Create BLE services  
- `configure_characteristic(uuid, service_uuid, properties)` - Create characteristics with properties
- `configure_characteristic_read(uuid, service_uuid, value)` - Set up read operations
- `get_service_info(uuid)` - Query service information
- `get_characteristic_info(characteristic_uuid, service_uuid)` - Query characteristic details
- `start_advertisement(allow_multi_connect)` - Start BLE advertising (auto-configures on first call)
- `notify_characteristic_value(address, address_type, characteristic_uuid, service_uuid, value)` - Send notifications

### New Commands (Added 2025)

#### `configure_profile(profile: BLEProfile)`
Configures the BLE device using a predefined profile, which restarts the server with all previously configured services and characteristics.

```python
from plugin_host.comms import USBHostDevice
import plugin_host.protocol_pb2 as protocol_pb2

with USBHostDevice() as device:
    device.configure_profile(protocol_pb2.BleProfile.Custom)
    print("Custom profile configured")
```

**Supported Profiles:**
- `protocol_pb2.BleProfile.Custom` - Uses existing service/characteristic definitions
- `protocol_pb2.BleProfile.HeartRateMonitor` - Heart rate monitoring profile
- `protocol_pb2.BleProfile.BatteryService` - Battery service profile
- `protocol_pb2.BleProfile.DeviceInformation` - Device information profile

## USBHostDevice Configuration

The `USBHostDevice` class can be initialized with optional parameters:

```python
from plugin_host.comms import USBHostDevice

# Default initialization
device = USBHostDevice()

# Custom configuration
device = USBHostDevice(
    vendor_id=0x1234,           # USB vendor ID (default: USB_VENDOR_ID)
    product_id=0x5678,          # USB product ID (default: USB_PRODUCT_ID)  
    default_command_delay=0.2   # Default delay between commands in seconds (default: DEFAULT_COMMAND_DELAY)
)
```

**Parameters:**
- `vendor_id`: USB vendor ID of the plugin device
- `product_id`: USB product ID of the plugin device  
- `default_command_delay`: Default delay in seconds between commands to ensure proper device communication

**Note:** The `default_command_delay` should be at least > 0.1 seconds to give the plugin sufficient time for processing commands.

## Usage Examples

### Basic Usage
```python
from plugin_host.comms import USBHostDevice
import plugin_host.protocol_pb2 as protocol_pb2

with USBHostDevice() as device:
    # Configure peripheral
    device.configure_peripheral("MyDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
    
    # Add service
    device.configure_service(0x1800)
    
    # Add characteristic
    device.configure_characteristic(0x2A00, 0x1800, [protocol_pb2.BleProperties.Read])
    
    # Start advertising (auto-configures profile on first call)
    device.start_advertisement()
```

### Using New Commands
```python
from plugin_host.comms import USBHostDevice
import plugin_host.protocol_pb2 as protocol_pb2

with USBHostDevice() as device:
    # Configure initial setup
    device.configure_peripheral("Demo", [0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
    device.configure_service(0x180F)  # Battery Service
    
    # Apply profile configuration
    device.configure_profile(protocol_pb2.BleProfile.Custom)  # Apply configuration
```

## Testing

Run all tests:
```bash
pytest tests/ -v
```

Run specific test:
```bash
pytest tests/test_plugin_host.py::test_new_commands_serialization -v
```

Current test status: **90 tests passing** (protobuf-only implementation)

## Examples

All examples are located in the `examples/` folder:

- `examples/example_usage.py` - Complete integration examples including the configure_profile command
- `examples/example_listening.py` - USB data listening examples