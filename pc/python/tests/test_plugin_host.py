import attrs2bin;
from enum import Enum
from collections import namedtuple
from plugin_host.types import *

def test_type_serialization() -> None:
    cmd = HostCommandConfigurePeripheral(uuid="123", name="Default peripheral")
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigurePeripheral)
    assert cmd == deserialized, "Peripheral configuration serialization"

    cmd = HostCommandConfigureService()
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigureService)
    assert cmd == deserialized, "Host service configuration deserialization"

    enum = PluginDataSendType.Notify
    serialized = attrs2bin.serialize(enum)
    deserialized = attrs2bin.deserialize(serialized, PluginDataSendType)    
    assert enum is deserialized, "Host data send type enum deserialization"

    data = PluginData(src_id="123",data=bytes([0,1,2]), send_type=PluginDataSendType.Notify)
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginData)
    assert data == deserialized, "Host data transmission"
