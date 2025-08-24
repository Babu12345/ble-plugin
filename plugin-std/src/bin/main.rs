use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use device_cherry::CdcAcmDevice;
use esp_idf_svc::{
    hal::{
        gpio::{OutputPin, PinDriver},
        prelude::Peripherals,
    },
    nvs::{EspNvsPartition, NvsDefault},
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use plugin_nvc::{namespace, namespaces::ConfigNamespace};
use plugin_state_machine_std::PluginStateMachine;
use plugin_std::errors::{PluginError, Result};

// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_default_partition: EspNvsPartition<NvsDefault> =
        EspNvsPartition::<NvsDefault>::take().unwrap();

    let _nvs = namespace::<ConfigNamespace>(nvs_default_partition)
        .map_err(|_| PluginError::UsbDeviceInitError("Failed to configure NVS namespace"))?;

    let peripherals = Peripherals::take().map_err(|_| PluginError::PeripheralsUnavailable)?;

    let indicator = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio21.downgrade_output())
            .map_err(|_| PluginError::GpioInitError("GPIO21"))?,
    ));

    indicator
        .lock()
        .map_err(|_| PluginError::GpioOperationError("Failed to lock GPIO"))?
        .set_high()
        .map_err(|_| PluginError::GpioOperationError("Failed to set GPIO low"))?;

    let usb_device = CdcAcmDevice::new()
        .init(0, ESP_USBD_BASE)
        .map_err(|_| PluginError::UsbDeviceInitError("Failed to initialize USB device"))?
        .set_dtr(0, false)
        .sleep(Duration::from_millis(500));

    std::thread::scope(|scope| {
        let usb_processors = usb_device
            .processors(scope, 20, (Duration::from_millis(10), 10))
            .unwrap();
        scope.spawn(
            PluginStateMachine::new(usb_processors.0, usb_processors.1, indicator).runner_fn(),
        );
    });

    Ok(())
}
