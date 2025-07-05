//! Processor functions. Should be run via threads for proper operation and not with async await
use esp_idf_sys::{
    TickType_t,
    host::{
        ESP_INTR_FLAG_LEVEL1, ESP_OK, USB_HOST_LIB_EVENT_FLAGS_ALL_FREE,
        USB_HOST_LIB_EVENT_FLAGS_NO_CLIENTS, cdc_acm_dev_hdl_t, cdc_acm_host_close,
        cdc_acm_host_data_tx_blocking, cdc_acm_host_desc_print, cdc_acm_host_dev_event_data_t,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_DEVICE_DISCONNECTED,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_ERROR,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_NETWORK_CONNECTION,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_SERIAL_STATE, cdc_acm_host_device_config_t,
        cdc_acm_host_install, cdc_acm_host_line_coding_get, cdc_acm_host_line_coding_set,
        cdc_acm_host_open, cdc_acm_host_set_control_line_state, cdc_acm_line_coding_t,
        usb_host_config_t, usb_host_device_free_all, usb_host_install, usb_host_lib_handle_events,
    },
};
use lib_utils::MatchSliceLengths;

use std::{
    ffi::c_void,
    ptr,
    sync::{
        OnceLock,
        mpsc::{Receiver, SyncSender},
    },
    thread::Scope,
    time::Duration,
};

use log::{error, info, trace, warn};

use crate::constants::*;

// TODO: Return a custom struct to control how data is being channeled to and from the usb interface.
// This should help to facilitate the defined API.
pub type T = [u8; 512];

pub static FROM_USB_SENDER: OnceLock<SyncSender<T>> = OnceLock::new();

unsafe fn lib_task() {
    let mut event_flags = 0;
    let timeout = TickType_t::from_be(USB_LIB_EVENT_MAX_DELAY);

    info!("USB host library task initiated");
    loop {
        unsafe { usb_host_lib_handle_events(timeout, &mut event_flags) };

        if event_flags & USB_HOST_LIB_EVENT_FLAGS_NO_CLIENTS != 0 {
            let res = unsafe { usb_host_device_free_all() };
            if res != ESP_OK {
                error!("Unable to free devices")
            } else {
                warn!("No clients connected");
            }
        }

        if event_flags & USB_HOST_LIB_EVENT_FLAGS_ALL_FREE != 0 {
            warn!("All clients freed")
        }
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn data_rx_handle(data: *const u8, data_len: usize, _args: *mut c_void) -> bool {
    let data = unsafe { core::slice::from_raw_parts(data, data_len) }.match_size(0);

    trace!("Data received: {:?}", String::from_utf8(Vec::from(&data)));

    match FROM_USB_SENDER.get().unwrap().try_send(data) {
        Ok(_) => {}
        Err(std::sync::mpsc::TrySendError::Full(_)) => {
            warn!(
                "Receive buffer is full. You must ingest in order to receive additional information."
            );
        }
        Err(std::sync::mpsc::TrySendError::Disconnected(_)) => {
            error!("Disconnect error");
        }
    }
    true
}

#[unsafe(no_mangle)]
#[allow(non_upper_case_globals, non_snake_case)]
unsafe extern "C" fn event_handle(
    event: *const cdc_acm_host_dev_event_data_t,
    _user_context: *mut c_void,
) {
    let event_val = unsafe { *event };
    match event_val.type_ {
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_ERROR => {
            error!("CDC error {} occurred", unsafe { event_val.data.error })
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_DEVICE_DISCONNECTED => {
            warn!("Device suddenly disconnected");
            let res = unsafe { cdc_acm_host_close(event_val.data.cdc_hdl) };
            if res != ESP_OK {
                error!("Failed to close connection")
            }
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_SERIAL_STATE => {
            info!("Serial state notification {:#04x}", unsafe {
                event_val.data.serial_state.val
            })
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_NETWORK_CONNECTION | _ => {
            error!("Unsupported CDC event {}", event_val.type_)
        }
    }
}

pub unsafe fn start_usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>) {
    let host_config = usb_host_config_t {
        skip_phy_setup: false,
        intr_flags: ESP_INTR_FLAG_LEVEL1 as i32,
        enum_filter_cb: None,
    };
    info!("Starting the host");
    let res = unsafe { usb_host_install(&host_config) };
    if res != ESP_OK {
        panic!("Unable to install the usb host");
    }
    scope.spawn(|| unsafe { lib_task() });

    info!("Installing the CDC-ACM host driver");
    let res = unsafe { cdc_acm_host_install(ptr::null()) };

    if res != ESP_OK {
        panic!("Unable to install the usb host");
    }
}

pub unsafe fn process_usb_cdc_host<'a>(receiver: Receiver<T>) {
    let config = cdc_acm_host_device_config_t {
        connection_timeout_ms: CONNECTION_TIMEOUT_MS,
        out_buffer_size: TX_BUFFER_SIZE,
        in_buffer_size: RX_BUFFER_SIZE,
        user_arg: ptr::null_mut(),
        event_cb: Some(event_handle),
        data_cb: Some(data_rx_handle),
    };
    info!(
        "Opening CDC ACM device {:#04x}:{:#04x}",
        USB_DEVICE_VID, USB_DEVICE_PID
    );
    let mut cdc_device_handler: cdc_acm_dev_hdl_t = ptr::null_mut();

    'wait_for_connection: loop {
        let res = unsafe {
            cdc_acm_host_open(
                USB_DEVICE_VID,
                USB_DEVICE_PID,
                0,
                &config,
                &mut cdc_device_handler,
            )
        };

        std::thread::sleep(Duration::from_millis(100));

        if res != ESP_OK {
            continue 'wait_for_connection;
        }
        break 'wait_for_connection;
    }

    // Print the device description to stdout
    unsafe { cdc_acm_host_desc_print(cdc_device_handler) };

    'set_configs: loop {
        let mut line_coding: cdc_acm_line_coding_t = Default::default();
        line_coding.dwDTERate = DEFAULT_DW_DTE_RATE;

        let res = unsafe { cdc_acm_host_line_coding_set(cdc_device_handler, &line_coding) };

        if res != ESP_OK {
            error!("Error setting line coding data");
            continue;
        }

        let res = unsafe { cdc_acm_host_line_coding_get(cdc_device_handler, &mut line_coding) };

        if res != ESP_OK {
            error!("Error getting line coding data");
            continue;
        }

        if line_coding.dwDTERate != DEFAULT_DW_DTE_RATE {
            panic!("Line coding set incorrectly")
        }

        info!("Line coding successfully set: {:?}", line_coding);

        let res = unsafe { cdc_acm_host_set_control_line_state(cdc_device_handler, true, false) };
        if res != ESP_OK {
            error!("Error setting control line data");
            continue;
        }
        break 'set_configs;
    }

    loop {
        let data = match receiver.recv() {
            Ok(data) => data,
            Err(e) => {
                info!("Error occurred {e}");
                continue;
            }
        };

        let res = unsafe {
            cdc_acm_host_data_tx_blocking(
                cdc_device_handler,
                data.as_ptr(),
                data.len(),
                TX_TIMEOUT_MS,
            )
        };

        if res != ESP_OK {
            error!("Error sending data to the CDC ACM device");
            continue;
        }
        trace!("Data transmitted: {:?}", data);
    }
}
