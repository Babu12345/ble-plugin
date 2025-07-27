import attr
import attrs2bin;
import usb.core
import usb.util
from enum import Enum
from collections import namedtuple
import plugin_host
from plugin_host.types import *

# Communicate between the host (PC) and the usb plugin