## Description
Project for a BLE plugin device set. Contains implementation for the host and plugin side processing.
Where possible, it also contains std and no_std implementations of the host and plugin code.

## Engineers
Babuabel Wanyeki (babs@wanyekitech.com)

## Links
https://docs.esp-rs.org/book/writing-your-own-application/generate-project/esp-generate.html 

## Business Docs
https://docs.google.com/document/d/1Dux7SiKq3yMgd7yeh_1pGXjGcVbrisn82CYdyfDYuJs/edit?tab=t.0


## Useful commands

### Find all usb devices connected to the computer
ls /dev/tty.*

### Monitor usb serial port. Useful when the main usb port is busy / programmed to operate differently
cargo espmonitor <SERIAL_DEVICE_PATH>

### Generate new esp-idf binary project (you can also just copy and paste from an existing project)
<!-- https://docs.espressif.com/projects/rust/book/writing-your-own-application/generate-project/index.html#esp-idf-template -->
cargo generate <test_project>

### Upload esp code to a board. Usually only needs to be done once. You can then make sure to source permanently by adding a line to the source file
espup install
. /Users/babuwanyeki/export-esp.sh or . $HOME/export-esp.sh
source ~/.zprofile


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