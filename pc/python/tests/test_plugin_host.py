import attr
import attrs2bin;
import usb.core
import usb.util
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

    enum = HostDataSendType.Notify
    serialized = attrs2bin.serialize(enum)
    deserialized = attrs2bin.deserialize(serialized, HostDataSendType)    
    assert enum is deserialized, "Host data send type enum deserialization"

    cmd = HostData(src_id="123",data=bytes([0,1,2]), send_type=HostDataSendType.Notify)
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostData)
    assert cmd == deserialized, "Host data transmission"
