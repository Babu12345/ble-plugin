
import attrs2bin
import attr
from enum import Enum

# Links: https://realpython.com/python-enum/. Python enum custom values
# https://stackoverflow.com/questions/35567724/how-to-define-custom-properties-in-enumeration-in-python-javascript-like
# Add serializers here: https://github.com/fvicent/attrs2bin/blob/master/attrs2bin/serializers.py


@attr.s(auto_attribs=True)
class HostCommandConfigurePeripheral:
    uuid: str
    name: str

@attr.s(auto_attribs=True)
class HostCommandConfigureService:
    pass

class PluginDataSendType(Enum):
    Notify: attrs2bin.U8 = 0
    Write: attrs2bin.U8 = 1
    Read: attrs2bin.U8 = 2
    

@attr.s(auto_attribs=True)
class PluginData:
    src_id: str
    send_type: PluginDataSendType
    data: bytes