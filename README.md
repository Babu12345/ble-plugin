## Description
Project for a BLE plugin device set. Contains implementation for the host and plugin side processing.
Where possible, it also contains std and no_std implementations of the host and plugin code.

## Engineers
Babuabel Wanyeki (babs@wanyekitech.com)

## Links
https://docs.esp-rs.org/book/writing-your-own-application/generate-project/esp-generate.html 

## Business Docs
https://docs.google.com/document/d/1Dux7SiKq3yMgd7yeh_1pGXjGcVbrisn82CYdyfDYuJs/edit?tab=t.0

## Code Generation

This project includes an automatic code generation system to maintain consistency between Rust and Python protocol implementations.

### Generate Python Types from Rust Protocol

```bash
# Generate Python types from Rust protocol definitions
./scripts/generate-python-types.sh
```

This script:
- Parses the Rust protocol library (`protocol/src/`)
- Generates equivalent Python code in `pc/python/plugin_host/generated_types.py`
- Ensures MessageTypeId ranges are consistent (host: 0x01-0x7F, plugin: 0x80-0xFF)
- Provides comprehensive test validation with 41 tests

For detailed documentation, see:
- **Script Usage**: [`scripts/README.md`](scripts/README.md)
- **Code Generator**: [`codegen/README.md`](codegen/README.md)

### Testing the Code Generator

```bash
cd codegen
cargo test                    # Run all 41 tests
cargo test --lib              # Unit tests
cargo test --test validation_tests  # Validation tests
```

## Useful commands

### Find all usb devices connected to the computer
For serial devices: ls /dev/tty.*

or

For all usb devices and their names / product details: system_profiler SPUSBDataType

### Monitor usb serial port. Useful when the main usb port is busy / programmed to operate differently
cargo espmonitor <SERIAL_DEVICE_PATH>

### Generate new esp-idf binary project (you can also just copy and paste from an existing project)
<!-- https://docs.espressif.com/projects/rust/book/writing-your-own-application/generate-project/index.html#esp-idf-template -->
cargo generate esp-rs/esp-idf-template cargo (then you can configure the name and everything else from the template)

Make sure you add 
# https://github.com/esp-rs/esp-idf-sys/blob/master/BUILD-OPTIONS.md#esp_idf_tools_install_dir-esp_idf_tools_install_dir
ESP_IDF_TOOLS_INSTALL_DIR = { value = "global" }

to the [env] section of the config.toml file

### Upload esp code to a board. Usually only needs to be done once. You can then make sure to source permanently by adding a line to the source file
espup install

. /Users/babuwanyeki/export-esp.sh or . $HOME/export-esp.sh

source ~/.zprofile


### Print in rust tests
cargo test -- --nocapture


## python (also in the pc/python readme)

### Create a virtual environment
python3 -m venv /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python

### Activate the virtual environment
source /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python/bin/activate

### Installations
pip install git+https://github.com/Babu12345/attrs2bin
pip install pyusb
pip install pytest
then add these lines to pytest.ini
```
[pytest]
pythonpath = .
```