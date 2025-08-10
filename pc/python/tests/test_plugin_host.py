import attrs2bin
import uuid as uuid_module
from enum import Enum
from collections import namedtuple
from plugin_host.comms import parse_uuid_u16
from plugin_host.generated_types import *

def test_type_serialization() -> None:
    cmd = HostCommandConfigurePeripheral(addr=[0x12, 0x30, 0x00, 0x00, 0x00, 0x00], name="Default peripheral")
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigurePeripheral)
    assert cmd == deserialized, "Peripheral configuration serialization"

    cmd = HostCommandConfigureService(uuid=0x4560)
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigureService)
    assert cmd == deserialized, "Host service configuration deserialization"

    enum = PluginDataSendType.Notify
    serialized = attrs2bin.serialize(enum)
    deserialized = attrs2bin.deserialize(serialized, PluginDataSendType)    
    assert enum is deserialized, "Host data send type enum deserialization"

    data = PluginData(send_type=PluginDataSendType.Notify, src_addr=[0x12, 0x30, 0x00, 0x00, 0x00, 0x00], src_addr_type=BluetoothAddressType.Public, data=bytes([0,1,2]))
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginData)
    assert data == deserialized, "Host data transmission"

    data = PluginServiceInfoResponse(
        service_uuid=0x7890,
        exists=True,
        characteristic_uuids= [
            0x1111,
            0x2222
        ],
    )
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginServiceInfoResponse)
    assert data == deserialized, "Host data transmission"