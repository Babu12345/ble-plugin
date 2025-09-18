use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use esp_idf_svc::hal::{
    gpio::{OutputPin, PinDriver},
    prelude::Peripherals,
};
use esp_idf_sys::cherry_host::ESP_USBH_BASE;
use host_cherry::CdcAcmHostDevice;
use plugin_host_std::errors::{PluginHostError, Result};
use plugin_nvs::EspNvsDefaultPartition;
use plugin_state_machine_std::PluginStateMachine;
use protocol::devices::{plugin::PluginProcessor, WriteThrottleInfo};
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_default_partition = EspNvsDefaultPartition::take().unwrap();
    let peripherals = Peripherals::take().map_err(|_| PluginHostError::PeripheralsUnavailable)?;

    let indicator = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio21.downgrade_output())
            .map_err(|_| PluginHostError::GpioInitError("GPIO21"))?,
    ));

    indicator
        .lock()
        .map_err(|_| PluginHostError::GpioOperationError("Failed to lock GPIO"))?
        .set_high()
        .map_err(|_| PluginHostError::GpioOperationError("Failed to set GPIO low"))?;

    let device = CdcAcmHostDevice::new()
        .init(0, ESP_USBH_BASE)
        .unwrap()
        .sleep(Duration::from_secs(1));
    std::thread::scope(|scope| {
        let processors = device
            .processors(
                scope,
                300,
                Default::default(),
                WriteThrottleInfo::default().set_delay(Duration::from_micros(500)),
            )
            .unwrap();

        scope.spawn(
            PluginStateMachine::new(processors.0, processors.1, indicator, nvs_default_partition)
                .unwrap()
                .runner_fn(),
        );
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
