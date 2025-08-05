use std::time::Duration;

use device_cherry::CdcAcmDevice;
use esp32_nimble::BLEDevice;
use esp_idf_svc::hal::{
    gpio::{AnyOutputPin, OutputPin, PinDriver},
    prelude::Peripherals,
};
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use lib_utils::mk_static;
use plugin_state_machine_std::PluginStateMachine;
use plugin_std::errors::{PluginError, Result};
// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().map_err(|_| PluginError::PeripheralsUnavailable)?;

    let indicator = mk_static!(
        PinDriver<'static, AnyOutputPin, esp_idf_svc::hal::gpio::Output>,
        PinDriver::output(peripherals.pins.gpio21.downgrade_output())
            .map_err(|_| PluginError::GpioInitError("GPIO21"))?
    );

    let usb_device = CdcAcmDevice::new()
        .init(0, ESP_USBD_BASE)
        .map_err(|_| PluginError::UsbDeviceInitError("Failed to initialize USB device"))?
        .set_dtr(0, true)
        .sleep(Duration::from_millis(500));

    std::thread::scope(move |scope| {
        let usb_processors = usb_device.processors(scope, 20).unwrap();

        scope.spawn(
            PluginStateMachine::new(
                usb_processors.0,
                usb_processors.1,
                BLEDevice::take(),
                indicator,
            )
            .runner_fn(),
        );
    });

    Ok(())
}
