
import attrs2bin
import attr
from enum import Enum
from typing import List, Optional

# Links: https://realpython.com/python-enum/. Python enum custom values
# https://stackoverflow.com/questions/35567724/how-to-define-custom-properties-in-enumeration-in-python-javascript-like
# Add serializers here: https://github.com/fvicent/attrs2bin/blob/master/attrs2bin/serializers.py

# Constants
MAX_NAME_SIZE = 32
MAX_PROPERTIES = 4
MAX_CHARACTERISTICS_PER_SERVICE = 16

# Message protocol constants
MESSAGE_MAGIC = 0xDEAD
MESSAGE_MAGIC_BYTES = 2
MESSAGE_TYPE_ID_BYTES = 1
DATA_BYTES_LENGTH_IN_BYTES = 2
MESSAGE_HEADER_SIZE = MESSAGE_MAGIC_BYTES + MESSAGE_TYPE_ID_BYTES + DATA_BYTES_LENGTH_IN_BYTES  # 5 bytes

class MessageTypeId(Enum):
    """Message type identifiers for command discrimination."""
    # Host commands
    HostCommandConfigurePeripheral = 0x01
    HostCommandConfigureService = 0x02
    HostCommandConfigureCharacteristic = 0x03
    HostCommandConfigureCharacteristicRead = 0x04
    HostCommandGetServiceInfo = 0x05
    HostCommandGetCharacteristicInfo = 0x06
    HostCommandStartAdvertisement = 0x07
    HostCommandNotifyCharacteristicValue = 0x08
    # Plugin responses
    PluginData = 0x10
    PluginConfigurationError = 0x11
    PluginServiceInfoResponse = 0x12
    PluginCharacteristicInfoResponse = 0x13


# Host types

@attr.s(auto_attribs=True)
class HostCommandConfigurePeripheral:
    """Host command to configure a peripheral device.
    
    Attributes:
        name: Peripheral name (max 32 characters)
        uuid: Peripheral UUID as string
    """
    name: str
    uuid: str

@attr.s(auto_attribs=True)
class HostCommandConfigureService:
    """Host command to configure a service.
    
    Attributes:
        uuid: Service UUID as string
    """
    uuid: str

class BLEProperties(Enum):
    """Properties enumeration for BLE characteristics.
    
    Values:
        READ: Read property
        WRITE: Write property  
        WriteNoRsp: Write without response property
        NOTIFY: Notify property
        INDICATE: Indicate property
    """
    READ = 0
    WRITE = 1
    WriteNoRsp = 2
    NOTIFY = 3
    INDICATE = 4

@attr.s(auto_attribs=True)
class HostCommandConfigureCharacteristic:
    """Host command to configure a characteristic.
    
    Attributes:
        uuid: Characteristic UUID as string
        service_uuid: Service UUID this characteristic belongs to
        properties: List of BLE properties (max 4 properties per characteristic)
    """
    uuid: str
    service_uuid: str
    properties: List[BLEProperties]

@attr.s(auto_attribs=True)
class HostCommandConfigureCharacteristicRead:
    """Host command to configure characteristic read operation.
    
    Attributes:
        uuid: Characteristic UUID as string
        service_uuid: Service UUID this characteristic belongs to
        value: Read value as bytes (max 32 bytes)
    """
    uuid: str
    service_uuid: str
    value: bytes

@attr.s(auto_attribs=True)
class HostCommandGetServiceInfo:
    """Host command to get service information.
    
    Attributes:
        uuid: Service UUID as string
    """
    uuid: str

@attr.s(auto_attribs=True)
class HostCommandGetCharacteristicInfo:
    """Host command to get characteristic information.
    
    Attributes:
        characteristic_uuid: Characteristic UUID as string
        service_uuid: Service UUID this characteristic belongs to
    """
    characteristic_uuid: str
    service_uuid: str

@attr.s(auto_attribs=True)
class HostCommandStartAdvertisement:
    """Host command to start advertisement.
    
    Attributes:
        allow_multi_connect: Allow multiple central connections
    """
    allow_multi_connect: bool

class BluetoothAddressType(Enum):
    """Bluetooth Device address type enumeration.
    
    Values:
        Public: Public address
        Random: Random address
        PublicID: Public ID address
        RandomID: Random ID address
    """
    Public = 0
    Random = 1
    PublicID = 2
    RandomID = 3

@attr.s(auto_attribs=True)
class HostCommandNotifyCharacteristicValue:
    """Host command to notify characteristic value.
    
    Attributes:
        address: Device Address as 6-byte array
        address_type: Address type
        characteristic_uuid: Characteristic UUID as string
        service_uuid: Service UUID this characteristic belongs to
        value: Value to notify as bytes (max 32 bytes)
    """
    address: bytes  # 6 bytes
    address_type: BluetoothAddressType
    characteristic_uuid: str
    service_uuid: str
    value: bytes

# Plugin types

class PluginDataSendType(Enum):
    """Represents the send type of the data.
    
    Was it due to a write event (central -> peripheral), notify event 
    (peripheral -> central), or read attempt (central -> peripheral). 
    Depending on which, a response might be expected or sent.
    
    Values:
        Notify: Notified from the central bluetooth device
        Read: Read attempt from the central bluetooth device
        Write: Written from the central bluetooth device
    """
    Notify: attrs2bin.U8 = 0
    Read: attrs2bin.U8 = 1
    Write: attrs2bin.U8 = 2

@attr.s(auto_attribs=True)
class PluginData:
    """Plugin data structure.
    
    Attributes:
        src_id: Source peripheral id that this data is originating from
        send_type: Send type of the data
        data: Actual command data as bytes
    """
    src_id: str
    data: bytes
    send_type: PluginDataSendType

class PluginConfigurationError(Enum):
    """Represents errors that can occur during plugin configuration.
    
    Values:
        PeripheralNameTooLong: The peripheral name is too long
        InvalidPeripheralUuid: The peripheral UUID is invalid
        InvalidServiceUuid: The service UUID is invalid
        InvalidCharacteristicUuid: The characteristic UUID is invalid
        AdvertisementWithoutPeripheralConfiguration: Advertisement without proper peripheral configuration
        ServiceWithoutPeripheralConfiguration: Service without proper peripheral configuration
        CharacteristicWithoutServiceConfiguration: Characteristic without proper service configuration
    """
    PeripheralNameTooLong: attrs2bin.U8 = 0
    InvalidPeripheralUuid: attrs2bin.U8 = 1
    InvalidServiceUuid: attrs2bin.U8 = 2
    InvalidCharacteristicUuid: attrs2bin.U8 = 3
    AdvertisementWithoutPeripheralConfiguration: attrs2bin.U8 = 4
    ServiceWithoutPeripheralConfiguration: attrs2bin.U8 = 5
    CharacteristicWithoutServiceConfiguration = 6

@attr.s(auto_attribs=True)
class PluginServiceInfoResponse:
    """Service information response.
    
    Attributes:
        service_uuid: Service UUID as string
        characteristic_uuids: List of characteristic UUIDs in this service (max 16)
        exists: Whether the service exists
    """
    service_uuid: str
    characteristic_uuids: List[str]
    exists: bool

@attr.s(auto_attribs=True)
class PluginCharacteristicInfoResponse:
    """Characteristic information response.
    
    Attributes:
        characteristic_uuid: Characteristic UUID as string
        service_uuid: Service UUID this characteristic belongs to
        properties: List of BLE properties (max 4)
        exists: Whether the characteristic exists
    """
    characteristic_uuid: str
    service_uuid: str
    properties: List[BLEProperties]
    exists: bool

# Map message types to their type IDs
MESSAGE_TYPE_MAP = {
    HostCommandConfigurePeripheral: MessageTypeId.HostCommandConfigurePeripheral,
    HostCommandConfigureService: MessageTypeId.HostCommandConfigureService,
    HostCommandConfigureCharacteristic: MessageTypeId.HostCommandConfigureCharacteristic,
    HostCommandConfigureCharacteristicRead: MessageTypeId.HostCommandConfigureCharacteristicRead,
    HostCommandGetServiceInfo: MessageTypeId.HostCommandGetServiceInfo,
    HostCommandGetCharacteristicInfo: MessageTypeId.HostCommandGetCharacteristicInfo,
    HostCommandStartAdvertisement: MessageTypeId.HostCommandStartAdvertisement,
    HostCommandNotifyCharacteristicValue: MessageTypeId.HostCommandNotifyCharacteristicValue,
    PluginData: MessageTypeId.PluginData,
    PluginConfigurationError: MessageTypeId.PluginConfigurationError,
    PluginServiceInfoResponse: MessageTypeId.PluginServiceInfoResponse,
    PluginCharacteristicInfoResponse: MessageTypeId.PluginCharacteristicInfoResponse,
}

# Reverse map for type ID to message type
TYPE_ID_TO_MESSAGE_TYPE = {
    MessageTypeId.HostCommandConfigurePeripheral: HostCommandConfigurePeripheral,
    MessageTypeId.HostCommandConfigureService: HostCommandConfigureService,
    MessageTypeId.HostCommandConfigureCharacteristic: HostCommandConfigureCharacteristic,
    MessageTypeId.HostCommandConfigureCharacteristicRead: HostCommandConfigureCharacteristicRead,
    MessageTypeId.HostCommandGetServiceInfo: HostCommandGetServiceInfo,
    MessageTypeId.HostCommandGetCharacteristicInfo: HostCommandGetCharacteristicInfo,
    MessageTypeId.HostCommandStartAdvertisement: HostCommandStartAdvertisement,
    MessageTypeId.HostCommandNotifyCharacteristicValue: HostCommandNotifyCharacteristicValue,
    MessageTypeId.PluginData: PluginData,
    MessageTypeId.PluginConfigurationError: PluginConfigurationError,
    MessageTypeId.PluginServiceInfoResponse: PluginServiceInfoResponse,
    MessageTypeId.PluginCharacteristicInfoResponse: PluginCharacteristicInfoResponse,
}