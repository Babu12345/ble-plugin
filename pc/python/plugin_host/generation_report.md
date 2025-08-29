# Protocol Code Generation Report

This report summarizes the Python code generated from the Rust protocol library.

## Constants Generated
- MESSAGE_MAGIC = 0xDEAD (u16)
- MAX_NAME_SIZE = 30 (usize)
- DEFAULT_PACKET_SIZE = 64 (usize)
- MAX_PROPERTIES = 4 (usize)
- MAX_CHARACTERISTICS_PER_SERVICE = 16 (usize)

## Enums Generated
- MessageTypeId (16 variants)
- BLEProperties (5 variants)
- BluetoothAddressType (4 variants)
- BLEProfile (4 variants)
- PluginDataSendType (3 variants)
- PluginConfigurationError (7 variants)

## Structs Generated
- HostCommandConfigurePeripheral (2 fields)
- HostCommandConfigurePeripheralSecurity (1 fields)
- HostCommandConfigureService (1 fields)
- HostCommandConfigureCharacteristic (3 fields)
- HostCommandConfigureCharacteristicRead (3 fields)
- HostCommandGetServiceInfo (1 fields)
- HostCommandGetCharacteristicInfo (2 fields)
- HostCommandStartAdvertisement (1 fields)
- HostCommandStopAdvertisement (0 fields)
- HostCommandNotifyCharacteristicValue (5 fields)
- HostCommandConfigureProfile (1 fields)
- PluginData (6 fields)
- PluginServiceInfoResponse (3 fields)
- PluginCharacteristicInfoResponse (4 fields)
- PluginAuthenticationCompletedResponse (3 fields)

## Usage

Replace the existing types.py file with generated_types.py, or carefully merge
the generated definitions into your existing code.

Generated at: 2025-08-29 02:30:07 UTC
