
## Create a virtual environment
python3 -m venv /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python

## Activate the virtual environment
source /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python/bin/activate

# Installations
pip install git+https://github.com/Babu12345/attrs2bin
pip install pyusb
pip install libusb
pip install pytest

brew install libusb # Make sure that this is run

then add these lines to pytest.ini
```
[pytest]
pythonpath = .
```

# Running tests
pytest <DIRECTORY_OF_TESTS>

# BLE Plugin Python Library

This Python library provides a high-level interface for communicating with BLE plugin devices over USB.

## Available Commands

### Core Commands

- `configure_peripheral(name, addr)` - Configure BLE peripheral with name and address
- `configure_service(uuid)` - Create BLE services  
- `configure_characteristic(uuid, service_uuid, properties)` - Create characteristics with properties
- `configure_characteristic_read(uuid, service_uuid, value)` - Set up read operations
- `get_service_info(uuid)` - Query service information
- `get_characteristic_info(characteristic_uuid, service_uuid)` - Query characteristic details
- `start_advertisement(allow_multi_connect)` - Start BLE advertising
- `notify_characteristic_value(address, address_type, characteristic_uuid, service_uuid, value)` - Send notifications

### New Commands (Added 2025)

#### `clear_all_services()`
Clears all configured BLE services and characteristics, resetting the device to a clean state.

```python
from plugin_host.comms import USBHostDevice

with USBHostDevice() as device:
    device.clear_all_services()
    print("All services cleared")
```

#### `configure_profile(profile: BLEProfile)`
Configures the BLE device using a predefined profile, which restarts the server with all previously configured services and characteristics.

```python
from plugin_host.comms import USBHostDevice
from plugin_host.generated_types import BLEProfile

with USBHostDevice() as device:
    device.configure_profile(BLEProfile.Custom)
    print("Custom profile configured")
```

**Supported Profiles:**
- `BLEProfile.Custom` - Uses existing service/characteristic definitions

## Usage Examples

### Basic Usage
```python
from plugin_host.comms import USBHostDevice
from plugin_host.generated_types import BLEProperties

with USBHostDevice() as device:
    # Configure peripheral
    device.configure_peripheral("MyDevice", [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC])
    
    # Add service
    device.configure_service("0x1800")
    
    # Add characteristic
    device.configure_characteristic("0x2A00", "0x1800", [BLEProperties.READ])
    
    # Start advertising
    device.start_advertisement()
```

### Using New Commands
```python
with USBHostDevice() as device:
    # Configure initial setup
    device.configure_peripheral("Demo", [0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
    device.configure_service("0x180F")  # Battery Service
    
    # Clear everything and start fresh
    device.clear_all_services()
    
    # Reconfigure
    device.configure_service("0x1800")  # Generic Access
    device.configure_profile(BLEProfile.Custom)  # Apply configuration
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

Current test status: **55 tests passing**

## Examples

All examples are located in the `examples/` folder:

- `examples/example_usage.py` - Basic integration examples
- `examples/example_new_commands.py` - Comprehensive demonstration of new commands  
- `examples/example_listening.py` - USB data listening examples