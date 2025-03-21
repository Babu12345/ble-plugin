use core::str;
use std::{ffi::c_void, ptr, thread::Scope, time::Duration};

use ble_plugin::utils::{
    CONNECTION_TIMEOUT_MS, RX_BUFFER_SIZE, TX_BUFFER_SIZE, TX_TIMEOUT_MS, USB_DEVICE_PID,
    USB_DEVICE_VID, USB_LIB_EVENT_MAX_DELAY,
};
use esp_idf_svc::hal::{
    prelude::Peripherals,
    spi::{
        config::{Config, DriverConfig},
        SpiDeviceDriver, SpiDriver,
    },
    units::Hertz,
};
use esp_idf_sys::{
    host::{
        cdc_acm_dev_hdl_t, cdc_acm_host_close, cdc_acm_host_data_tx_blocking,
        cdc_acm_host_desc_print, cdc_acm_host_dev_event_data_t,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_DEVICE_DISCONNECTED,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_ERROR,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_NETWORK_CONNECTION,
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_SERIAL_STATE, cdc_acm_host_device_config_t,
        cdc_acm_host_install, cdc_acm_host_open, usb_host_config_t, usb_host_device_free_all,
        usb_host_install, usb_host_lib_handle_events, ESP_INTR_FLAG_LEVEL1, ESP_OK,
        USB_HOST_LIB_EVENT_FLAGS_ALL_FREE, USB_HOST_LIB_EVENT_FLAGS_NO_CLIENTS,
    },
    TickType_t,
};
use log::{error, info, warn};

const EXAMPLE_STRING_SEND: &str = "Hello";

unsafe fn lib_task() {
    let mut event_flags = 0;
    let timeout = TickType_t::from_be(USB_LIB_EVENT_MAX_DELAY);

    info!("USB host library task initiated");
    loop {
        usb_host_lib_handle_events(timeout, &mut event_flags);

        if event_flags & USB_HOST_LIB_EVENT_FLAGS_NO_CLIENTS != 0 {
            let res = usb_host_device_free_all();
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

#[no_mangle]
unsafe extern "C" fn data_rx_handle(data: *const u8, data_len: usize, args: *mut c_void) -> bool {
    let args = args as *mut u8;
    let input_args = core::slice::from_raw_parts(args, 10);
    info!("Input arguments: {:?}", input_args);
    let data = core::slice::from_raw_parts(data, data_len);
    info!("Data received: {:?}", data);
    true
}

#[no_mangle]
#[allow(non_upper_case_globals)]
unsafe extern "C" fn event_handle(
    event: *const cdc_acm_host_dev_event_data_t,
    _user_context: *mut c_void,
) {
    let event_val = *event;
    match event_val.type_ {
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_ERROR => {
            info!("CDC error {} occurred", event_val.data.error)
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_DEVICE_DISCONNECTED => {
            info!("Device suddenly disconnected");
            let res = cdc_acm_host_close(event_val.data.cdc_hdl);
            if res != ESP_OK {
                error!("Failed to close connection")
            }
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_SERIAL_STATE => {
            info!(
                "Serial state notification {:#04x}",
                event_val.data.serial_state.val
            )
        }
        cdc_acm_host_dev_event_t_CDC_ACM_HOST_NETWORK_CONNECTION | _ => {
            error!("Unsupported CDC event {}", event_val.type_)
        }
    }
}

unsafe fn start_usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>) {
    let host_config = usb_host_config_t {
        skip_phy_setup: false,
        intr_flags: ESP_INTR_FLAG_LEVEL1 as i32,
        enum_filter_cb: None,
    };
    info!("Starting the host");
    let res = usb_host_install(&host_config);
    if res != ESP_OK {
        panic!("Unable to install the usb host");
    }
    scope.spawn(|| lib_task());

    info!("Installing the CDC-ACM host driver");
    let res = cdc_acm_host_install(ptr::null());

    if res != ESP_OK {
        panic!("Unable to install the usb host");
    }
}

unsafe fn process_usb_cdc_host<'a>(mut _spi: SpiDeviceDriver<'a, SpiDriver<'a>>) {
    let mut data = [2u8; 10];

    let config = cdc_acm_host_device_config_t {
        connection_timeout_ms: CONNECTION_TIMEOUT_MS,
        out_buffer_size: TX_BUFFER_SIZE,
        in_buffer_size: RX_BUFFER_SIZE,
        user_arg: data.as_mut_ptr() as *mut c_void,
        event_cb: Some(event_handle),
        data_cb: Some(data_rx_handle),
    };
    info!(
        "Opening CDC ACM device {:#04x}:{:#04x}",
        USB_DEVICE_VID, USB_DEVICE_PID
    );
    loop {
        let mut cdc_device_handler: cdc_acm_dev_hdl_t = ptr::null_mut();

        let res = cdc_acm_host_open(
            USB_DEVICE_VID,
            USB_DEVICE_PID,
            0,
            &config,
            &mut cdc_device_handler,
        );

        if res != ESP_OK {
            error!("Error opening the CDC ACM device. Error code: {}", res);
            continue;
        }

        // Print the device description to stdout
        cdc_acm_host_desc_print(cdc_device_handler);
        std::thread::sleep(Duration::from_millis(100));

        let res = cdc_acm_host_data_tx_blocking(
            cdc_device_handler,
            EXAMPLE_STRING_SEND.as_ptr(),
            EXAMPLE_STRING_SEND.len(),
            TX_TIMEOUT_MS,
        );

        // Test sending data
        if res != ESP_OK {
            error!("Error sending data to the CDC ACM device");
            continue;
        }

        return;
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let mosi = peripherals.pins.gpio9;
    let miso = peripherals.pins.gpio8;
    let sclk = peripherals.pins.gpio7;
    let cs = peripherals.pins.gpio1;

    let spi: SpiDeviceDriver<'_, SpiDriver<'_>> = SpiDeviceDriver::new_single(
        peripherals.spi2,
        sclk,
        mosi,
        Some(miso),
        Some(cs),
        &DriverConfig::default(),
        &Config::default().baudrate(Hertz(80_000_000)),
    )
    .unwrap();

    unsafe {
        std::thread::scope(|s| {
            start_usb_host(s);
            s.spawn(move || process_usb_cdc_host(spi));
        });
    }
}
