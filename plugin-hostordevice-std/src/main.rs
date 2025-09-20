use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use device_cherry::CdcAcmDevice;
use esp_idf_svc::hal::{
    gpio::{OutputPin, PinDriver},
    prelude::Peripherals,
};
use esp_idf_sys::{cherry_device::ESP_USBD_BASE, cherry_host::ESP_USBH_BASE};
use host_cherry::CdcAcmHostDevice;
use plugin_hostordevice_std::errors::{PluginError, Result};
use plugin_hostordevice_std::resolver::USBTypeResolver;
use plugin_nvs::EspNvsDefaultPartition;
use plugin_state_machine_std::PluginStateMachine;
use protocol::devices::{plugin::PluginProcessor, WriteThrottleInfo};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let nvs_default_partition = EspNvsDefaultPartition::take().unwrap();
    let peripherals = Peripherals::take().map_err(|_| PluginError::PeripheralsUnavailable)?;

    let mut input_pin = PinDriver::input(peripherals.pins.gpio9)
        .map_err(|_| PluginError::GpioInitError("GPIO9"))?;
    input_pin
        .set_pull(esp_idf_svc::hal::gpio::Pull::Down)
        .map_err(|_| PluginError::GpioOperationError("Failed to pull GPIO9 down"))?;

    let indicator = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio21.downgrade_output())
            .map_err(|_| PluginError::GpioInitError("GPIO21"))?,
    ));

    indicator
        .lock()
        .map_err(|_| PluginError::GpioOperationError("Failed to lock GPIO21"))?
        .set_high()
        .map_err(|_| PluginError::GpioOperationError("Failed to set GPIO21 high"))?;

    std::thread::scope(|scope| {
        let processors = match input_pin.get_level().into() {
            USBTypeResolver::UsbHost => CdcAcmHostDevice::new()
                .init(0, ESP_USBH_BASE)
                .map_err(|_| PluginError::UsbInitError(USBTypeResolver::UsbHost))?
                .sleep(Duration::from_secs(1))
                .processors(
                    scope,
                    300,
                    Default::default(),
                    WriteThrottleInfo::default().set_delay(Duration::from_micros(500)),
                )
                .map_err(|_| PluginError::ProcessorInitError(USBTypeResolver::UsbHost)),
            USBTypeResolver::UsbDevice => CdcAcmDevice::new()
                .init(0, ESP_USBD_BASE)
                .map_err(|_| PluginError::UsbInitError(USBTypeResolver::UsbDevice))?
                .set_dtr(0, false)
                .sleep(Duration::from_millis(500))
                .processors(scope, 300, Default::default(), Default::default())
                .map_err(|_| PluginError::ProcessorInitError(USBTypeResolver::UsbHost)),
        }?;

        scope.spawn(
            PluginStateMachine::new(processors.0, processors.1, indicator, nvs_default_partition)
                .unwrap()
                .runner_fn(),
        );

        Ok(())
    })?;

    Ok(())
}
