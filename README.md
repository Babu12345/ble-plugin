## Description

A comprehensive BLE (Bluetooth Low Energy) plugin framework supporting bidirectional communication between host devices and plugin modules. This project provides a robust protocol implementation with automatic code generation to ensure consistency across multiple programming languages.

### Key Features

- **Dual-Architecture Support**: Complete implementations for both host and plugin sides
- **Cross-Platform Compatibility**: Standard library (`std`) and embedded (`no_std`) implementations
- **Protocol Consistency**: Automated code generation maintains synchronization between Rust and Python implementations
- **Comprehensive Testing**: Extensive test coverage with 41+ validation tests for protocol integrity
- **Message Type Management**: Structured message ID ranges (host: 0x01-0x7F, plugin: 0x80-0xFF)
- **ESP32 Integration**: Ready-to-use templates and utilities for ESP-IDF based hardware

### Architecture

The project is organized into distinct components:
- **Protocol Core**: Rust-based protocol definitions and I/O operations
- **Host Implementation**: Python-based host-side communication layer
- **Plugin Framework**: Embedded-friendly plugin implementations
- **Code Generation**: Automated tooling for cross-language protocol consistency
- **Testing Suite**: Comprehensive validation across all components

## Engineers
Babuabel Wanyeki (babs@wanyekitech.com)

## Links
https://docs.esp-rs.org/book/writing-your-own-application/generate-project/esp-generate.html 

## Business Docs
https://docs.google.com/document/d/1Dux7SiKq3yMgd7yeh_1pGXjGcVbrisn82CYdyfDYuJs/edit?tab=t.0

## Code Generation

This project includes an automatic code generation system to maintain consistency between Rust and Python protocol implementations.

### Protocol Definitions

This project uses **Protocol Buffers (protobuf)** exclusively for all communication between host and plugin components. The protocol definitions are centralized and automatically generate consistent type definitions across platforms.

```bash
# Protocol definitions are in protobuf format
# See protocol/src/ for .proto files and generated code
```

Key protocol features:
- Uses Protocol Buffers for reliable, cross-platform serialization
- Ensures MessageTypeId ranges are consistent (host: 0x01-0x7F, plugin: 0x80-0xFF)
- Type-safe message definitions with automatic code generation
- Comprehensive test validation with 90+ tests

For detailed protocol documentation, see:
- **Protocol Library**: [`protocol/README.md`](protocol/README.md)
- **Python Implementation**: [`pc/python/README.md`](pc/python/README.md)

## Testing

This project includes a comprehensive test script that can test all Rust crates and Python packages in the project.

### Run All Tests

```bash
# Test everything (Rust + Python + Compilation check)
./test_all.sh

# Test only Rust crates
./test_all.sh rust

# Check compilation for selected crates
./test_all.sh compile

# Test only Python packages  
./test_all.sh python

# Show help
./test_all.sh --help
```

The test script will:
- **Rust**: Test core Rust crates with comprehensive test suites
- **Compilation**: Check compilation for additional selected crates (protocol_io, host-std, plugin variants, etc.)
- **Python**: Test Python packages in their virtual environments, focusing only on project code (not site-packages)
- Provide colored output showing successes and failures
- Exit with error code if any tests fail

### Tested Components

**Rust Crates:**
- `lib_utils` - Utility functions
- `protocol_io` - Protocol I/O operations  
- `protocol` - Core protocol definitions (17 tests)
- `codegen` - Code generation tools (41 tests total)

**Python Packages:**
- `pc/python` - Host-side Python implementation (90 tests, protobuf-only)

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

https://github.com/esp-rs/esp-idf-sys/blob/master/BUILD-OPTIONS.md#esp_idf_tools_install_dir-esp_idf_tools_install_dir

ESP_IDF_TOOLS_INSTALL_DIR = { value = "global" }

to the [env] section of the config.toml file

### Upload esp code to a board. Usually only needs to be done once. You can then make sure to source permanently by adding a line to the source file
espup install

. /Users/babuwanyeki/export-esp.sh or . $HOME/export-esp.sh

source ~/.zprofile


### Print in rust tests
cargo test -- --nocapture


## Python (also in the pc/python readme)
### Generate the protobuf files for serialization/deserialization in python
1. First install protoc via `brew install protobuf`
2. Run `protoc --python_out=../pc/python/plugin_host/ protocol.proto`

### Create a virtual environment
`python3 -m venv /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python`

### Activate the virtual environment
`source /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python/bin/activate`

### Installations - for the pc/python host libraries
`pip install protobuf`
`pip install pyusb`
`pip install pytest`

then add these lines to pytest.ini

brew install libusb # Make sure that this is run
```
[pytest]
pythonpath = .
```

## Importing private repos
### [Create a new key](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent)
1. Generate a ssh key using ED25519 cryptography `ssh-keygen -t ed25519 -C "bwanyeki12@gmail.com"`
2. Refresh the ssh agent with the new key `eval "$(ssh-agent -s)"`
3. `open ~/.ssh/config` or `touch ~/.ssh/config`(if it doesn't exist) and paste
```
Host github.com
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile ~/.ssh/id_ed25519
```
4. Add to apple keychain `ssh-add --apple-use-keychain ~/.ssh/id_ed25519`

### [Copy to github account](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account#adding-a-new-ssh-key-to-your-account)
1. pbcopy < ~/.ssh/id_ed25519.pub