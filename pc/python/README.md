
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
- `start_advertisement(allow_multi_connect)` - Start BLE advertising (auto-configures on first call)
- `notify_characteristic_value(address, address_type, characteristic_uuid, service_uuid, value)` - Send notifications

### New Commands (Added 2025)

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

**Note:** The `default_command_delay` should be at least > 0.01 seconds to give the plugin sufficient time for processing commands.

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
    
    # Start advertising (auto-configures profile on first call)
    device.start_advertisement()
```

### Using New Commands
```python
with USBHostDevice() as device:
    # Configure initial setup
    device.configure_peripheral("Demo", [0x11, 0x22, 0x33, 0x44, 0x55, 0x66])
    device.configure_service("0x180F")  # Battery Service
    
    # Apply profile configuration
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

- `examples/example_usage.py` - Complete integration examples including the configure_profile command
- `examples/example_listening.py` - USB data listening examples