import attr
import attrs2bin;
import usb.core
import usb.util

@attr.s(auto_attribs=True)
class HostCommand:
    uuid: str

@attr.s(auto_attribs=True)
class BulkHostData:
    commands: list[HostCommand]

def main() -> None:
    for i in range(10):
        print("Hello {i}")

main()