import attrs2bin
import uuid as uuid_module
from enum import Enum
from collections import namedtuple
from plugin_host.generated_types import *

def uuid_str_to_bytes(uuid_str: str) -> bytes:
    """Convert UUID string to bytes"""
    # Parse UUID and get bytes
    uuid_obj = uuid_module.UUID(uuid_str)
    return uuid_obj.bytes

def test_type_serialization() -> None:
    cmd = HostCommandConfigurePeripheral(addr=[0x12, 0x30, 0x00, 0x00, 0x00, 0x00], name="Default peripheral")
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigurePeripheral)
    assert cmd == deserialized, "Peripheral configuration serialization"

    cmd = HostCommandConfigureService(uuid=uuid_str_to_bytes("45600000-0000-0000-0000-000000000000"))
    serialized = attrs2bin.serialize(cmd)
    deserialized = attrs2bin.deserialize(serialized, HostCommandConfigureService)
    assert cmd == deserialized, "Host service configuration deserialization"

    enum = PluginDataSendType.Notify
    serialized = attrs2bin.serialize(enum)
    deserialized = attrs2bin.deserialize(serialized, PluginDataSendType)    
    assert enum is deserialized, "Host data send type enum deserialization"

    data = PluginData(send_type=PluginDataSendType.Notify, src_id=uuid_str_to_bytes("12300000-0000-0000-0000-000000000000"),data=bytes([0,1,2]))
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginData)
    assert data == deserialized, "Host data transmission"

    data = PluginServiceInfoResponse(
        service_uuid=uuid_str_to_bytes("78900000-0000-0000-0000-000000000000"),
        exists=True,
        characteristic_uuids= [
            uuid_str_to_bytes("11111111-1111-1111-1111-111111111111"),
            uuid_str_to_bytes("22222222-2222-2222-2222-222222222222")
        ],
    )
    serialized = attrs2bin.serialize(data)
    deserialized = attrs2bin.deserialize(serialized, PluginServiceInfoResponse)
    assert data == deserialized, "Host data transmission"