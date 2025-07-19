//! USB device class for the esp-idf hal interace
//! Taking example for https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/usb_serial.rs for the final product of
//! how this will be called and referenced in code.
#![allow(static_mut_refs)]
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use esp_idf_sys::cherry_device::{
    cdc_line_coding, usb_descriptor, usbd_add_endpoint, usbd_add_interface, usbd_cdc_acm_init_intf,
    usbd_cdc_acm_set_line_coding, usbd_desc_register, usbd_endpoint, usbd_ep_start_read,
    usbd_ep_start_write, usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_CONFIGURED, usbd_event_type_USBD_EVENT_CONNECTED,
    usbd_event_type_USBD_EVENT_DISCONNECTED, usbd_event_type_USBD_EVENT_RESET,
    usbd_event_type_USBD_EVENT_RESUME, usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_SUSPEND, usbd_get_ep_mps, usbd_initialize, usbd_interface,
    CDC_ACM_DESCRIPTOR_LEN, USB_2_0, USB_2_1, USB_BULK_EP_MPS_FS, USB_BULK_EP_MPS_HS,
    USB_CONFIG_BUS_POWERED, USB_DBG_LOG, USB_DEVICE_CLASS_CDC, USB_DEVICE_CLASS_MISC,
    USB_SPEED_FULL, USB_SPEED_HIGH,
};
use esp_idf_sys::vTaskDelay;

use crate::mk_static;
mod utils;
use utils::{
    cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
    device_qualifier_descriptor_init, other_speed_descriptor_init, CDC_MAX_MPS,
};

use std::ptr;
use std::sync::{LazyLock, Mutex};
const CDC_IN_EP: u8 = 0x81;
const CDC_OUT_EP: u8 = 0x02;
const CDC_INT_EP: u8 = 0x83; // 0x85
const USB_CONFIG_SIZE: u32 = 9 + CDC_ACM_DESCRIPTOR_LEN;
const USBD_VID: u16 = 0xFFFF;
const USBD_PID: u16 = 0xFFFF;
const USBD_MAX_POWER: u32 = 100; // 2mA * 100 = 100 mA

static mut READ_BUFFER_LOCKER: [u8; 2048] = [0; 2048];

// unsafe extern "C" {
//     unsafe static mut ep_tx_busy_flag: bool;

// }

// /// Strong reference to the function defined in C
// #[unsafe(no_mangle)]
// #[allow(unused_variables)]
// pub extern "C" fn usbd_cdc_acm_set_dtr(busid: u8, intf: u8, dtr: bool) {
//     unsafe { std::ptr::write_volatile(&raw mut dtr_enable, dtr) }
// }

// /// Strong reference to the cdc acm stopper defined in C
// #[unsafe(no_mangle)]
// #[allow(unused_variables)]
// pub unsafe extern "C" fn cdc_acm_data_send_with_dtr_test(busid: u8) {
//     if unsafe { core::ptr::read_volatile(&raw const ep_tx_busy_flag) } {
//         unsafe { core::ptr::write_volatile(&raw mut ep_tx_busy_flag, true) };
//         let mut data = [0; 2048];
//         usbd_ep_start_write(busid, CDC_IN_EP as u8, data.as_mut_ptr(), 2048);
//         while unsafe { core::ptr::read_volatile(&raw const ep_tx_busy_flag) } {}
//     }
// }

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L19
static DEVICE_DESCRIPTOR: [u8; 18] = [
    18,   // bLength
    0x01, // bDescriptorType (Device)
    0x00,
    0x02, // bcdUSB (USB 2.0)
    0xEF, // bDeviceClass
    0x02, // bDeviceSubClass
    0x01, // bDeviceProtocol
    64,   // bMaxPacketSize0
    (USBD_VID & 0xFF) as u8,
    (USBD_VID >> 8) as u8, // idVendor
    (USBD_PID & 0xFF) as u8,
    (USBD_PID >> 8) as u8, // idProduct
    0x00,
    0x01, // bcdDevice
    0x01, // iManufacturer
    0x02, // iProduct
    0x03, // iSerialNumber
    0x01, // bNumConfigurations
];

static CONFIG_DESCRIPTOR: [u8; 67] = [
    // Configuration descriptor
    9,    // bLength
    0x02, // bDescriptorType (Configuration)
    67,
    0,                    // wTotalLength
    0x02,                 // bNumInterfaces
    0x01,                 // bConfigurationValue
    0x00,                 // iConfiguration
    0x80,                 // bmAttributes (USB_CONFIG_BUS_POWERED)
    USBD_MAX_POWER as u8, // bMaxPower
    // CDC ACM Interface descriptors (simplified)
    9,
    0x04,
    0x00,
    0x00,
    0x01,
    0x02,
    0x02,
    0x01,
    0x00, // Interface 0
    5,
    0x24,
    0x00,
    0x10,
    0x01, // CDC Header
    5,
    0x24,
    0x01,
    0x00,
    0x01, // CDC Call Management
    4,
    0x24,
    0x02,
    0x02, // CDC ACM
    5,
    0x24,
    0x06,
    0x00,
    0x01, // CDC Union
    7,
    0x05,
    CDC_INT_EP as u8,
    0x03,
    0x08,
    0x00,
    0xFF, // Interrupt endpoint
    9,
    0x04,
    0x01,
    0x00,
    0x02,
    0x0A,
    0x00,
    0x00,
    0x00, // Interface 1
    7,
    0x05,
    CDC_OUT_EP as u8,
    0x02,
    (CDC_MAX_MPS & 0xFF) as u8,
    (CDC_MAX_MPS >> 8) as u8,
    0x00, // OUT endpoint
    7,
    0x05,
    CDC_IN_EP as u8,
    0x02,
    (CDC_MAX_MPS & 0xFF) as u8,
    (CDC_MAX_MPS >> 8) as u8,
    0x00, // IN endpoint
];

static DEVICE_QUALITY_DESCRIPTOR: [u8; 10] = [
    10,   // bLength
    0x06, // bDescriptorType (Device Qualifier)
    0x00, 0x02, // bcdUSB
    0x00, // bDeviceClass
    0x00, // bDeviceSubClass
    0x00, // bDeviceProtocol
    0x40, // bMaxPacketSize0
    0x00, // bNumConfigurations
    0x00, // bReserved
];

static STRING_MANUFACTURER: &[u8] = b"CherryUSB\0";
static STRING_PRODUCT: &[u8] = b"CherryUSB CDC DEMO\0";
static STRING_SERIAL: &[u8] = b"2022123456\0";
static STRING_LANGID: &[u8] = b"\x09\x04\0";

// https://github.com/orangecms/RV-Debugger-BL702/blob/05739699b50a9235f8906bd80b4b8f7dd0c37e62/components/usb_stack/common/usb_def.h#L473
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn device_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_DESCRIPTOR.as_ptr() as *const u8
}

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L72
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn device_quality_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_QUALITY_DESCRIPTOR.as_ptr()
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn config_descriptor_callback(speed: u8) -> *const u8 {
    // log::info!("Event");
    // match speed as u32 {
    //     USB_SPEED_HIGH => CONFIG_DESCRIPTOR_HS.as_ptr() as *const u8,
    //     USB_SPEED_FULL => CONFIG_DESCRIPTOR_FS.as_ptr() as *const u8,
    //     _ => ptr::null(),
    // }
    CONFIG_DESCRIPTOR.as_ptr()
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn other_speed_descriptor_callback(speed: u8) -> *const u8 {
    // log::info!("Event");
    // match speed as u32 {
    //     USB_SPEED_HIGH => OTHER_SPEED_CONFIG_DESCRIPTOR_HS.as_ptr() as *const u8,
    //     USB_SPEED_FULL => OTHER_SPEED_CONFIG_DESCRIPTOR_FS.as_ptr() as *const u8,
    //     _ => ptr::null(),
    // }
    DEVICE_QUALITY_DESCRIPTOR.as_ptr()
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn string_descriptor_callback(_speed: u8, index: u8) -> *const u8 {
    match index {
        0 => STRING_LANGID.as_ptr() as *const u8,
        1 => STRING_MANUFACTURER.as_ptr() as *const u8,
        2 => STRING_PRODUCT.as_ptr() as *const u8,
        3 => STRING_SERIAL.as_ptr() as *const u8,
        _ => ptr::null(),
    }
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn usbd_cdc_acm_bulk_out(busid: u8, ep: u8, _nbytes: u32) {
    // log::info!("Event");
    let _res = usbd_ep_start_read(
        busid,
        CDC_OUT_EP as u8,
        READ_BUFFER_LOCKER.as_mut_ptr(),
        2048,
    );

    // log::info!("Data incoming: {:?}", READ_BUFFER_LOCKER);
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    // log::info!("Outgoing");
    let ep_mps = usbd_get_ep_mps(busid, ep) as u32;
    if (nbytes % ep_mps) == 0 && nbytes > 0 {
        /* send zlp */
        let _res = usbd_ep_start_write(busid, ep, ptr::null(), 0);
        return;
    }
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn usbd_event_handler(busid: u8, event: u8) {
    #[allow(non_upper_case_globals, non_snake_case)]
    match event as u32 {
        usbd_event_type_USBD_EVENT_RESET
        | usbd_event_type_USBD_EVENT_CONNECTED
        | usbd_event_type_USBD_EVENT_DISCONNECTED
        | usbd_event_type_USBD_EVENT_RESUME
        | usbd_event_type_USBD_EVENT_SUSPEND
        | usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP
        | usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP => {}
        usbd_event_type_USBD_EVENT_CONFIGURED => {
            let _res = usbd_ep_start_read(
                busid,
                CDC_OUT_EP as u8,
                READ_BUFFER_LOCKER.as_mut_ptr(),
                2048,
            );
            // log::info!("Event: {event}");
        }
        _ => {}
    }
}

/// test
pub type T = [u8; 256];

/// test
pub unsafe fn receive_usb_data(_sender: Sender<T>) {}

/// test
pub unsafe fn send_usb_data(_receiver: Receiver<T>) {}

/// Sending usb data
pub unsafe fn send_data(data: &mut [u8]) {
    let _res = usbd_ep_start_write(0, CDC_IN_EP as u8, data.as_mut_ptr(), 2048);
}

/// https://github.com/CherryUSB/cherryusb_esp32/tree/main/examples/device
pub unsafe fn cdc_init(busid: u8, reg_base: u32) {
    usbd_desc_register(
        busid,
        mk_static!(
            usb_descriptor,
            usb_descriptor {
                device_descriptor_callback: Some(device_descriptor_callback),
                config_descriptor_callback: Some(config_descriptor_callback),
                device_quality_descriptor_callback: Some(device_quality_descriptor_callback),
                other_speed_descriptor_callback: Some(other_speed_descriptor_callback),
                string_descriptor_callback: Some(string_descriptor_callback),
                ..Default::default()
            }
        ),
    );
    usbd_add_interface(
        busid,
        usbd_cdc_acm_init_intf(busid, mk_static!(usbd_interface, usbd_interface::default())),
    );
    usbd_add_interface(
        busid,
        usbd_cdc_acm_init_intf(busid, mk_static!(usbd_interface, usbd_interface::default())),
    );
    usbd_add_endpoint(
        busid,
        mk_static!(
            usbd_endpoint,
            usbd_endpoint {
                ep_addr: CDC_OUT_EP as u8,
                ep_cb: Some(usbd_cdc_acm_bulk_out),
            }
        ),
    );
    usbd_add_endpoint(
        busid,
        mk_static!(
            usbd_endpoint,
            usbd_endpoint {
                ep_addr: CDC_IN_EP as u8,
                ep_cb: Some(usbd_cdc_acm_bulk_in),
            }
        ),
    );
    let res = usbd_initialize(busid, reg_base as usize, Some(usbd_event_handler));

    if res < 0 {
        // log::error!("Failed to initialize the board");
    }
}
