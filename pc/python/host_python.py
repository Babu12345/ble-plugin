import attr
import attrs2bin;
import usb.core
import usb.util
from enum import Enum
from collections import namedtuple

# Links: https://realpython.com/python-enum/. Python enum custom values
# https://stackoverflow.com/questions/35567724/how-to-define-custom-properties-in-enumeration-in-python-javascript-like
# Add serializers here: https://github.com/fvicent/attrs2bin/blob/master/attrs2bin/serializers.py

@attr.s(auto_attribs=True)
class HostCommand:
    uuid: str
# TODO:  Create a serializer for these list types
@attr.s(auto_attribs=True)
class BulkHostCommand:
    commands: list[HostCommand]

@attr.s(auto_attribs=True)
class Sprite:
    name: str
    x: int
    y: list[int]

@attr.s(auto_attribs=True)
class Example:
    name: list[int]

def main() -> None:
    # bulk_cmd = BulkHostCommand(commands=[HostCommand(uuid="123")])
    # serialized = attrs2bin.serialize(bulk_cmd)
    # deserialized = attrs2bin.deserialize(serialized, BulkHostCommand)
    my_sprite = Sprite(name="My sprite", x=35, y=[70])
    serialized = attrs2bin.serialize(my_sprite)
    deserialized = attrs2bin.deserialize(serialized, Sprite)
    print(deserialized)

main()