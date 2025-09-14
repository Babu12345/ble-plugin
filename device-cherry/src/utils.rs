//! Util functions

use esp_idf_sys::cherry_device::{
    CDC_ABSTRACT_CONTROL_MODEL, CDC_COMMON_PROTOCOL_AT_COMMANDS, CDC_CS_INTERFACE,
    CDC_DATA_INTERFACE_CLASS, CDC_FUNC_DESC_ABSTRACT_CONTROL_MANAGEMENT,
    CDC_FUNC_DESC_CALL_MANAGEMENT, CDC_FUNC_DESC_HEADER, CDC_FUNC_DESC_UNION, CDC_V1_10,
    USB_DESCRIPTOR_TYPE_CONFIGURATION, USB_DESCRIPTOR_TYPE_DEVICE, USB_DESCRIPTOR_TYPE_ENDPOINT,
    USB_DESCRIPTOR_TYPE_INTERFACE, USB_DESCRIPTOR_TYPE_INTERFACE_ASSOCIATION,
    USB_DESCRIPTOR_TYPE_OTHER_SPEED, USB_DEVICE_CLASS_CDC, USB_STRING_MFC_INDEX,
    USB_STRING_PRODUCT_INDEX, USB_STRING_SERIAL_INDEX,
};
use protocol::DEFAULT_PACKET_SIZE;

/// Max packet size for bulk endpoints
pub const CDC_BULK_MPS: u32 = DEFAULT_PACKET_SIZE as _;

/// Max packet size for control endpoint (always 64 bytes for USB 2.0)
pub const CDC_CONTROL_MPS: u32 = 64;

// https://github.com/bekencorp/bk_idk/blob/650e754e12fe1e43c37ce2316a973668b033fd48/components/bk_usb/CherryUSB/common/usb_def.h#L628
/// Device descriptor
pub fn device_descriptor_init(
    bcd_usb: u32,
    b_device_class: u32,
    b_device_sub_class: u32,
    b_device_protocol: u32,
    id_vendor: u32,
    id_product: u32,
    bcd_device: u32,
    b_num_configurations: u32,
) -> [u8; 18] {
    [
        0x12,                       /* bLength */
        USB_DESCRIPTOR_TYPE_DEVICE, /* bDescriptorType */
        bcd_usb & 0xff,             /* bcdUSB */
        (bcd_usb >> 8) & 0xFF,      /* bcdUSB */
        b_device_class,             /* bDeviceClass */
        b_device_sub_class,         /* bDeviceSubClass */
        b_device_protocol,          /* bDeviceProtocol */
        CDC_CONTROL_MPS,            /* bMaxPacketSize */
        id_vendor & 0xff,           /* idVendor */
        (id_vendor >> 8) & 0xFF,    /* idVendor */
        id_product & 0xff,          /* idProduct */
        (id_product >> 8) & 0xFF,   /* idProduct */
        bcd_device & 0xff,          /* bcdDevice */
        (bcd_device >> 8) & 0xFF,   /* bcdDevice */
        USB_STRING_MFC_INDEX,       /* iManufacturer */
        USB_STRING_PRODUCT_INDEX,   /* iProduct */
        USB_STRING_SERIAL_INDEX,    /* iSerial */
        b_num_configurations,       /* bNumConfigurations */
    ]
    .map(|x| x as u8)
}

// https://github.com/bekencorp/bk_idk/blob/650e754e12fe1e43c37ce2316a973668b033fd48/components/bk_usb/CherryUSB/common/usb_def.h#L644
/// Device configuration descriptor
pub fn config_descriptor_init(
    w_total_length: u32,
    b_num_interfaces: u32,
    b_configuration_value: u32,
    bm_attributes: u32,
    b_max_power: u32,
) -> [u8; 9] {
    [
        0x09,                              /* bLength */
        USB_DESCRIPTOR_TYPE_CONFIGURATION, /* bDescriptorType */
        w_total_length & 0xff,             /* wTotalLength */
        (w_total_length >> 8) & 0xFF,      /* wTotalLength */
        b_num_interfaces,                  /* bNumInterfaces */
        b_configuration_value,             /* bConfigurationValue */
        0x00,                              /* iConfiguration */
        bm_attributes,                     /* bmAttributes */
        b_max_power / 2,                   /* bMaxPower */
    ]
    .map(|x| x as u8)
}

// https://github.com/MiSTle-Dev/bouffalo_sdk/blob/14f95db1a1c2cfc2e48e9a4c631d99bcbeeeafaa/components/usb/cherryusb/common/usb_def.h#L667
/// Device quality descriptor
pub fn device_quality_descriptor_init(
    bcd_usb: u32,
    b_device_class: u32,
    b_device_sub_class: u32,
    b_device_protocol: u32,
    b_num_configurations: u32,
) -> [u8; 10] {
    [
        0x0A,                              /* bLength */
        USB_DESCRIPTOR_TYPE_CONFIGURATION, /* bDescriptorType */
        bcd_usb & 0xff,                    /* bcdUSB */
        (bcd_usb >> 8) & 0xFF,             /* bcdUSB */
        b_device_class,                    /* bDeviceClass */
        b_device_sub_class,                /* bDeviceSubClass */
        b_device_protocol,                 /* bDeviceProtocol */
        CDC_CONTROL_MPS,                   /* bmAttributes */
        b_num_configurations,              /* bNumConfigurations */
        0x00,                              /* bReserved */
    ]
    .map(|x| x as u8)
}

// https://github.com/wdfk-prog/RT-Thread-Study/blob/919ba18009f95ddc74f3d6fd54ac7f7ef81139c0/42%20USB.md?plain=1#L1654
/// CDC ACM device descriptor
pub fn cdc_acm_descriptor_init(
    b_first_interface: u32,
    int_ep: u32,
    out_ep: u32,
    in_ep: u32,
    w_max_packet_size: u32,
    str_idx: u32,
) -> [u8; 66] {
    [
        0x08,                                      /* bLength */
        USB_DESCRIPTOR_TYPE_INTERFACE_ASSOCIATION, /* bDescriptorType */
        b_first_interface,                         /* bFirstInterface */
        0x02,                                      /* bInterfaceCount */
        USB_DEVICE_CLASS_CDC,                      /* bFunctionClass */
        CDC_ABSTRACT_CONTROL_MODEL,                /* bFunctionSubClass */
        CDC_COMMON_PROTOCOL_AT_COMMANDS,           /* bFunctionProtocol */
        0x00,                                      /* iFunction */
        0x09,                                      /* bLength */
        USB_DESCRIPTOR_TYPE_INTERFACE,             /* bDescriptorType */
        b_first_interface,                         /* bInterfaceNumber */
        0x00,                                      /* bAlternateSetting */
        0x01,                                      /* bNumEndpoints */
        USB_DEVICE_CLASS_CDC,                      /* bInterfaceClass */
        CDC_ABSTRACT_CONTROL_MODEL,                /* bInterfaceSubClass */
        CDC_COMMON_PROTOCOL_AT_COMMANDS,           /* bInterfaceProtocol */
        str_idx,                                   /* iInterface */
        0x05,                                      /* bLength */
        CDC_CS_INTERFACE,                          /* bDescriptorType */
        CDC_FUNC_DESC_HEADER,                      /* bDescriptorSubtype */
        CDC_V1_10 & 0xff,                          /* bcdCDC */
        (CDC_V1_10 >> 8) & 0xFF,                   /* bcdCDC */
        0x05,                                      /* bLength */
        CDC_CS_INTERFACE,                          /* bDescriptorType */
        CDC_FUNC_DESC_CALL_MANAGEMENT,             /* bDescriptorSubtype */
        0x00,                                      /* bmCapabilities */
        b_first_interface + 1,                     /* bDataInterface */
        0x04,                                      /* bLength */
        CDC_CS_INTERFACE,                          /* bDescriptorType */
        CDC_FUNC_DESC_ABSTRACT_CONTROL_MANAGEMENT, /* bDescriptorSubtype */
        0x02,                                      /* bmCapabilities */
        0x05,                                      /* bLength */
        CDC_CS_INTERFACE,                          /* bDescriptorType */
        CDC_FUNC_DESC_UNION,                       /* bDescriptorSubtype */
        b_first_interface,                         /* bMasterInterface */
        b_first_interface + 1,                     /* bSlaveInterface0 */
        0x07,                                      /* bLength */
        USB_DESCRIPTOR_TYPE_ENDPOINT,              /* bDescriptorType */
        int_ep,                                    /* bEndpointAddress */
        0x03,                                      /* bmAttributes */
        0x08,
        0x00,                            /* wMaxPacketSize */
        0x0a,                            /* bInterval */
        0x09,                            /* bLength */
        USB_DESCRIPTOR_TYPE_INTERFACE,   /* bDescriptorType */
        b_first_interface + 1,           /* bInterfaceNumber */
        0x00,                            /* bAlternateSetting */
        0x02,                            /* bNumEndpoints */
        CDC_DATA_INTERFACE_CLASS,        /* bInterfaceClass */
        0x00,                            /* bInterfaceSubClass */
        0x00,                            /* bInterfaceProtocol */
        0x00,                            /* iInterface */
        0x07,                            /* bLength */
        USB_DESCRIPTOR_TYPE_ENDPOINT,    /* bDescriptorType */
        out_ep,                          /* bEndpointAddress */
        0x02,                            /* bmAttributes */
        w_max_packet_size & 0xff,        /* wMaxPacketSize */
        (w_max_packet_size >> 8) & 0xFF, /* wMaxPacketSize */
        0x00,                            /* bInterval */
        0x07,                            /* bLength */
        USB_DESCRIPTOR_TYPE_ENDPOINT,    /* bDescriptorType */
        in_ep,                           /* bEndpointAddress */
        0x02,                            /* bmAttributes */
        w_max_packet_size & 0xff,        /* wMaxPacketSize */
        (w_max_packet_size >> 8) & 0xFF, /* wMaxPacketSize */
        0x00,
    ]
    .map(|x| x as u8)
}

// https://github.com/MiSTle-Dev/bouffalo_sdk/blob/14f95db1a1c2cfc2e48e9a4c631d99bcbeeeafaa/components/usb/cherryusb/common/usb_def.h#L667
/// Device other speed config descriptor
pub fn other_speed_config_descriptor(
    w_total_length: u32,
    b_num_interfaces: u32,
    b_configuration_value: u32,
    bm_attributes: u32,
    b_max_power: u32,
) -> [u8; 9] {
    [
        0x09,                            /* bLength */
        USB_DESCRIPTOR_TYPE_OTHER_SPEED, /* bDescriptorType */
        w_total_length & 0xff,           /* wTotalLength */
        (w_total_length >> 8) & 0xFF,    /* wTotalLength */
        b_num_interfaces,                /* bNumInterfaces */
        b_configuration_value,           /* bConfigurationValue */
        0x00,                            /* iConfiguration */
        bm_attributes,                   /* bmAttributes */
        b_max_power / 2,                 /* bMaxPower */
    ]
    .map(|x| x as u8)
}

#[macro_export]
/// Concat N arrays
macro_rules! concat_n_arrays {
    ($arr:expr) => { $arr };

    ($arr1:expr, $arr2:expr) => {{
        let a1 = $arr1;
        let a2 = $arr2;
        std::array::from_fn(|i| {
            if i < a1.len() { a1[i] } else { a2[i - a1.len()] }
        })
    }};

    ($first:expr, $($rest:expr),+) => {
        concat_arrays!(
            $first,
            concat_arrays!($($rest),+)
        )
    };
}
