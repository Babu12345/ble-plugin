//! Processors

//! USB device class for the esp-idf hal interace
//! Taking example for https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/usb_serial.rs for the final product of
//! how this will be called and referenced in code.

use std::sync::mpsc::{Receiver, Sender};

use esp_idf_sys::cherry_device::{
    CDC_ACM_DESCRIPTOR_LEN, USB_2_0, USB_BULK_EP_MPS_FS, USB_BULK_EP_MPS_HS,
    USB_CONFIG_BUS_POWERED, USB_DEVICE_CLASS_MISC, USB_SPEED_FULL, USB_SPEED_HIGH, usb_descriptor,
    usbd_add_endpoint, usbd_add_interface, usbd_cdc_acm_init_intf, usbd_desc_register,
    usbd_endpoint, usbd_ep_start_read, usbd_ep_start_write,
    usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP, usbd_event_type_USBD_EVENT_CONFIGURED,
    usbd_event_type_USBD_EVENT_CONNECTED, usbd_event_type_USBD_EVENT_DISCONNECTED,
    usbd_event_type_USBD_EVENT_RESET, usbd_event_type_USBD_EVENT_RESUME,
    usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP, usbd_event_type_USBD_EVENT_SUSPEND,
    usbd_get_ep_mps, usbd_initialize, usbd_interface,
};

use crate::utils;
use std::ptr;
use std::sync::{LazyLock, Mutex};
use utils::{
    cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
    device_qualifier_descriptor_init, other_speed_config_descriptor_init,
};
const CDC_IN_EP: u32 = 0x81;
const CDC_OUT_EP: u32 = 0x01;
const CDC_INT_EP: u32 = 0x83;
const USB_CONFIG_SIZE: u32 = 9 + CDC_ACM_DESCRIPTOR_LEN;
const USBD_VID: u16 = 0x32B7;
const USBD_PID: u16 = 0xFFFF;
const USBD_MAX_POWER: u32 = 50; // 2mA * 50 = 100 mA

static READ_BUFFER_LOCKER: Mutex<[u8; 2048]> = Mutex::new([0; 2048]);

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L19
static DEVICE_DESCRIPTOR: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [device_descriptor_init(
        USB_2_0,
        USB_DEVICE_CLASS_MISC,
        0x02,
        0x01,
        USBD_VID as u32,
        USBD_PID as u32,
        0x0100,
        0x01,
    )]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L23
static CONFIG_DESCRIPTOR_HS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        ),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_HS,
            0x02,
        ),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L28
static CONFIG_DESCRIPTOR_FS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        ),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_FS,
            0x02,
        ),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L33C22-L33C47
static DEVICE_QUALITY_DESCRIPTOR: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [device_qualifier_descriptor_init(
        USB_2_0,
        USB_DEVICE_CLASS_MISC,
        0x02,
        0x01,
        0x01,
    )]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L23
static OTHER_SPEED_CONFIG_DESCRIPTOR_HS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        other_speed_config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        ),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_HS,
            0x02,
        ),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L28
static OTHER_SPEED_CONFIG_DESCRIPTOR_FS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        other_speed_config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        ),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_FS,
            0x02,
        ),
    ]
    .concat()
});

static STRING_DESCRIPTOR: LazyLock<[&'static str; 4]> = LazyLock::new(|| {
    [
        std::str::from_utf8(&[0x09, 0x04]).unwrap(),
        "HPMicro",
        "HPMicro CDC DEMO",
        "2024051702",
    ]
});

// https://github.com/orangecms/RV-Debugger-BL702/blob/05739699b50a9235f8906bd80b4b8f7dd0c37e62/components/usb_stack/common/usb_def.h#L473
unsafe extern "C" fn device_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_DESCRIPTOR.as_slice().as_ptr() as *const u8
}

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L72
unsafe extern "C" fn device_quality_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_QUALITY_DESCRIPTOR.as_slice().as_ptr() as *const u8
}

unsafe extern "C" fn config_descriptor_callback(speed: u8) -> *const u8 {
    #[allow(non_snake_case)]
    match speed as u32 {
        USB_SPEED_HIGH => CONFIG_DESCRIPTOR_HS.as_slice().as_ptr() as *const u8,
        USB_SPEED_FULL => CONFIG_DESCRIPTOR_FS.as_slice().as_ptr() as *const u8,
        _ => ptr::null(),
    }
}

unsafe extern "C" fn other_speed_config_descriptor_callback(speed: u8) -> *const u8 {
    #[allow(non_snake_case)]
    match speed as u32 {
        USB_SPEED_HIGH => OTHER_SPEED_CONFIG_DESCRIPTOR_HS.as_slice().as_ptr() as *const u8,
        USB_SPEED_FULL => OTHER_SPEED_CONFIG_DESCRIPTOR_FS.as_slice().as_ptr() as *const u8,
        _ => ptr::null(),
    }
}

unsafe extern "C" fn string_descriptor_callback(_speed: u8, index: u8) -> *const u8 {
    match index >= STRING_DESCRIPTOR.len() as u8 {
        true => ptr::null(),
        false => STRING_DESCRIPTOR[index as usize].as_ptr() as *const u8,
    }
}

unsafe extern "C" fn usbd_cdc_acm_bulk_out(busid: u8, ep: u8, _nbytes: u32) {
    unsafe {
        usbd_ep_start_read(
            busid,
            ep,
            READ_BUFFER_LOCKER.lock().unwrap().as_mut_ptr(),
            usbd_get_ep_mps(busid, ep).into(),
        );
    }
}

unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    unsafe {
        if (nbytes % usbd_get_ep_mps(busid, ep) as u32) == 0 && nbytes > 0 {
            /* send zlp */
            usbd_ep_start_write(busid, ep, ptr::null(), 0);
        }
    }
}

unsafe extern "C" fn usbd_event_handler(busid: u8, event: u8) {
    #[allow(non_upper_case_globals, non_snake_case)]
    match event as u32 {
        usbd_event_type_USBD_EVENT_RESET
        | usbd_event_type_USBD_EVENT_CONNECTED
        | usbd_event_type_USBD_EVENT_DISCONNECTED
        | usbd_event_type_USBD_EVENT_RESUME
        | usbd_event_type_USBD_EVENT_SUSPEND
        | usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP
        | usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP => log::info!("Unknown event"),
        usbd_event_type_USBD_EVENT_CONFIGURED => unsafe {
            usbd_ep_start_read(
                busid,
                CDC_OUT_EP as u8,
                READ_BUFFER_LOCKER.lock().unwrap().as_mut_ptr(),
                usbd_get_ep_mps(busid, CDC_OUT_EP as u8).into(),
            );
        },
        _ => {}
    }
}

/// test
pub type T = [u8; 256];

/// test
pub unsafe fn receive_usb_data(_sender: Sender<T>) {}

/// test
pub unsafe fn send_usb_data(_receiver: Receiver<T>) {}

/// test
pub unsafe fn cdc_init(busid: u8, reg_base: u32) {
    let cdc_descriptor = usb_descriptor {
        device_descriptor_callback: Some(device_descriptor_callback),
        config_descriptor_callback: Some(config_descriptor_callback),
        device_quality_descriptor_callback: Some(device_quality_descriptor_callback),
        other_speed_descriptor_callback: Some(other_speed_config_descriptor_callback),
        string_descriptor_callback: Some(string_descriptor_callback),
        msosv1_descriptor: ptr::null(),
        msosv2_descriptor: ptr::null(),
        webusb_url_descriptor: ptr::null(),
        bos_descriptor: ptr::null(),
    };

    //     /*!< endpoint call back */
    let mut cdc_out_ep = usbd_endpoint {
        ep_addr: CDC_OUT_EP as u8,
        ep_cb: Some(usbd_cdc_acm_bulk_out),
    };

    //     /*!< endpoint call back */
    let mut cdc_in_ep = usbd_endpoint {
        ep_addr: CDC_IN_EP as u8,
        ep_cb: Some(usbd_cdc_acm_bulk_in),
    };

    let intf0: *mut usbd_interface = ptr::null_mut();
    let intf1: *mut usbd_interface = ptr::null_mut();

    unsafe {
        usbd_desc_register(busid, &cdc_descriptor);
        usbd_add_interface(busid, usbd_cdc_acm_init_intf(busid, intf0));
        usbd_add_interface(busid, usbd_cdc_acm_init_intf(busid, intf1));
        usbd_add_endpoint(busid, &mut cdc_out_ep);
        usbd_add_endpoint(busid, &mut cdc_in_ep);
        usbd_initialize(busid, reg_base as usize, Some(usbd_event_handler));
    }
}
