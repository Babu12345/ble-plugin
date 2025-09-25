use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use device_cherry::CdcAcmDevice;
use esp_idf_svc::hal::{
    gpio::{OutputPin, PinDriver},
    prelude::Peripherals,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use plugin_nvs::EspNvsDefaultPartition;
use plugin_state_machine_std::PluginStateMachine;
use plugin_std::errors::{PluginError, Result};
use protocol::devices::plugin::PluginProcessor;

// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_default_partition = EspNvsDefaultPartition::take().unwrap();
    let peripherals = Peripherals::take().map_err(|_| PluginError::PeripheralsUnavailable)?;

    let indicator = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio21.downgrade_output())
            .map_err(|_| PluginError::GpioInitError("GPIO21"))?,
    ));

    indicator
        .lock()
        .map_err(|_| PluginError::GpioOperationError("Failed to lock GPIO"))?
        .set_high()
        .map_err(|_| PluginError::GpioOperationError("Failed to set GPIO high"))?;

    let usb_device = CdcAcmDevice::new()
        .init(0, ESP_USBD_BASE)
        .map_err(|_| PluginError::UsbDeviceInitError("Failed to initialize USB device"))?
        .sleep(Duration::from_millis(500));
    std::thread::scope(|scope| {
        let processors = usb_device
            .processors(scope, 100, Default::default(), Default::default())
            .unwrap();
        scope.spawn(
            PluginStateMachine::new(processors.0, processors.1, indicator, nvs_default_partition)
                .unwrap()
                .runner_fn(),
        );
    });

    Ok(())
}
