
## Create a virtual environment
python3 -m venv /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python

## Activate the virtual environment
source /Users/babuwanyeki/Documents/Rusty/ble-plugin/pc/python/bin/activate

# Installations
pip install git+https://github.com/Babu12345/attrs2bin
pip install pyusb
pip install pytest
then add these lines to pytest.ini
```
[pytest]
pythonpath = .
```

# Running tests
pytest <DIRECTORY_OF_TESTS>