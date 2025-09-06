use std::time::Duration;

use esp_idf_sys::cherry_host::ESP_USBH_BASE;
use host_cherry::CdcAcmHost;
use protocol::devices::host::HostProcessor;
/**
 * General protocal is as follows:
 * There will be 2 tasks. One task is responsible for sending commands and data.
 * The other is responsible for processing such commands and data. The commands
 * and data bytes will each be structured and typesafe as well as the responses to
 * any of the commands. By using a channel we can also dictate how large the buffer should
 * be as well as make sure the order of the commands and data is what we expect.
 */
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let device = CdcAcmHost::new()
        .init(0, ESP_USBH_BASE)
        .unwrap()
        .sleep(Duration::from_millis(500));

    std::thread::scope(|scope| {
        let _processors = device
            .processors(scope, 20, (Duration::from_millis(10), 10))
            .unwrap();
    });

    Ok(())
} // See https://github.com/espressif/esp-idf/blob/v5.4.1/examples/peripherals/usb/host/cdc/cdc_acm_host/main/usb_cdc_example_main.c
