use device_cherry::CdcAcmDevice;
use esp32_nimble::BLEDevice;
use esp_idf_sys::cherry_device::ESP_USBD_BASE;
use plugin_state_machine_std::PluginStateMachine;

// Examples: https://github.com/taks/esp32-nimble/tree/main/examples
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let usb_device = CdcAcmDevice::new()
        .init(0, ESP_USBD_BASE)
        .unwrap()
        .set_dtr(0, true);

    std::thread::scope(|scope| {
        let usb_processors = usb_device.processors(scope, 20).unwrap();

        scope.spawn(
            PluginStateMachine::new(usb_processors.0, usb_processors.1, BLEDevice::take())
                .runner_fn(),
        );
    });
}
