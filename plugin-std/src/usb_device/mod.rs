//! USB device class for the esp-idf hal interace
//! Taking example for https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/usb_serial.rs for the final product of
//! how this will be called and referenced in code.

use std::sync::mpsc::{Receiver, Sender};

use esp_idf_sys::cherry_device::{
    cdc_line_coding, usb_descriptor, usbd_add_endpoint, usbd_add_interface, usbd_cdc_acm_init_intf,
    usbd_cdc_acm_set_line_coding, usbd_desc_register, usbd_endpoint, usbd_ep_start_read,
    usbd_ep_start_write, usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_CONFIGURED, usbd_event_type_USBD_EVENT_CONNECTED,
    usbd_event_type_USBD_EVENT_DISCONNECTED, usbd_event_type_USBD_EVENT_RESET,
    usbd_event_type_USBD_EVENT_RESUME, usbd_event_type_USBD_EVENT_SET_REMOTE_WAKEUP,
    usbd_event_type_USBD_EVENT_SUSPEND, usbd_initialize, usbd_interface, CDC_ACM_DESCRIPTOR_LEN,
    USB_2_0, USB_BULK_EP_MPS_FS, USB_BULK_EP_MPS_HS, USB_CONFIG_BUS_POWERED, USB_DEVICE_CLASS_CDC,
    USB_DEVICE_CLASS_MISC, USB_SPEED_FULL, USB_SPEED_HIGH,
};

use crate::mk_static;
mod utils;
use utils::{
    cdc_acm_descriptor_init, config_descriptor_init, device_descriptor_init,
    device_qualifier_descriptor_init, other_speed_descriptor_init, CDC_MAX_MPS,
};

use std::ptr;
use std::sync::{LazyLock, Mutex};
const CDC_IN_EP: u32 = 0x81;
const CDC_OUT_EP: u32 = 0x02;
const CDC_INT_EP: u32 = 0x85; // 0x85
const USB_CONFIG_SIZE: u32 = 9 + CDC_ACM_DESCRIPTOR_LEN;
const USBD_VID: u16 = 0xFFFF;
const USBD_PID: u16 = 0xFFFF;
const USBD_MAX_POWER: u32 = 50; // 2mA * 50 = 100 mA

static READ_BUFFER_LOCKER: Mutex<[u8; 2048]> = Mutex::new([0; 2048]);

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
const DEVICE_DESCRIPTOR: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [device_descriptor_init(
        USB_2_0,
        USB_DEVICE_CLASS_MISC, // USB_DEVICE_CLASS_CDC
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
const CONFIG_DESCRIPTOR_HS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        )
        .to_vec(),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_HS,
            0x02,
        )
        .to_vec(),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L28
const CONFIG_DESCRIPTOR_FS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        config_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        )
        .to_vec(),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_FS,
            0x02,
        )
        .to_vec(),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L33C22-L33C47
const DEVICE_QUALITY_DESCRIPTOR: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [device_qualifier_descriptor_init(
        USB_2_0,
        USB_DEVICE_CLASS_CDC,
        0x02,
        0x01,
        0x01,
    )]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L23
const OTHER_SPEED_CONFIG_DESCRIPTOR_HS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        other_speed_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        )
        .to_vec(),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_FS,
            0x02,
        )
        .to_vec(),
    ]
    .concat()
});

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L28
const OTHER_SPEED_CONFIG_DESCRIPTOR_FS: LazyLock<Vec<u32>> = LazyLock::new(|| {
    [
        other_speed_descriptor_init(
            USB_CONFIG_SIZE,
            0x02,
            0x01,
            USB_CONFIG_BUS_POWERED,
            USBD_MAX_POWER,
        )
        .to_vec(),
        cdc_acm_descriptor_init(
            0x00,
            CDC_INT_EP,
            CDC_OUT_EP,
            CDC_IN_EP,
            USB_BULK_EP_MPS_FS,
            0x02,
        )
        .to_vec(),
    ]
    .concat()
});

const STRING_DESCRIPTOR: &[&[u8]] = &[
    &[0x09, 0x04],
    b"HPMicro",
    b"HPMicro CDC DEMO",
    b"2024051702",
];

// https://github.com/orangecms/RV-Debugger-BL702/blob/05739699b50a9235f8906bd80b4b8f7dd0c37e62/components/usb_stack/common/usb_def.h#L473
unsafe extern "C" fn device_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_DESCRIPTOR.as_ptr() as *const u8
}

// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/device/cdc_acm/cdc_acm_vcom/src/cdc_acm.c#L72
unsafe extern "C" fn device_quality_descriptor_callback(_speed: u8) -> *const u8 {
    DEVICE_QUALITY_DESCRIPTOR.as_ptr() as *const u8
}

unsafe extern "C" fn config_descriptor_callback(speed: u8) -> *const u8 {
    match speed as u32 {
        USB_SPEED_HIGH => CONFIG_DESCRIPTOR_HS.as_ptr() as *const u8,
        USB_SPEED_FULL => CONFIG_DESCRIPTOR_FS.as_ptr() as *const u8,
        _ => ptr::null(),
    }
}

unsafe extern "C" fn other_speed_descriptor_callback(speed: u8) -> *const u8 {
    match speed as u32 {
        USB_SPEED_HIGH => OTHER_SPEED_CONFIG_DESCRIPTOR_HS.as_ptr() as *const u8,
        USB_SPEED_FULL => OTHER_SPEED_CONFIG_DESCRIPTOR_FS.as_ptr() as *const u8,
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
    let _res = usbd_ep_start_read(
        busid,
        ep,
        READ_BUFFER_LOCKER.lock().unwrap().as_mut_ptr(),
        2048,
    );

    // log::info!("Data incoming: {:?}", *READ_BUFFER_LOCKER.lock().unwrap());
}

#[unsafe(no_mangle)]
unsafe extern "C" fn usbd_cdc_acm_bulk_in(busid: u8, ep: u8, nbytes: u32) {
    log::info!("Outgoing");
    if (nbytes % CDC_MAX_MPS) == 0 && nbytes > 0 {
        /* send zlp */
        let _res = usbd_ep_start_write(busid, ep, ptr::null(), 0);
        return;
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
        | usbd_event_type_USBD_EVENT_CLR_REMOTE_WAKEUP => {}
        usbd_event_type_USBD_EVENT_CONFIGURED => {
            let _res = usbd_ep_start_read(
                busid,
                CDC_OUT_EP as u8,
                READ_BUFFER_LOCKER.lock().unwrap().as_mut_ptr(),
                2048,
            );
            log::info!("Event: {event}");
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
        usbd_cdc_acm_init_intf(busid, mk_static!(usbd_interface, Default::default())),
    );
    usbd_add_interface(
        busid,
        usbd_cdc_acm_init_intf(busid, mk_static!(usbd_interface, Default::default())),
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
    usbd_initialize(busid, reg_base as usize, Some(usbd_event_handler));
}
