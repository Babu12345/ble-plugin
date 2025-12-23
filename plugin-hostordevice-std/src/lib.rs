#![deny(missing_docs)]
// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Plugin device that can either function as a usb host or device

pub mod errors;
pub mod resolver;

/// Setup a custom panic handler
pub fn setup_custom_panic() {
    // Set up a panic handler that logs the panic and restarts the device
    std::panic::set_hook(Box::new(|panic_info| {
        log::error!("PANIC: {}", panic_info);
        // Give time for the log to be flushed
        std::thread::sleep(std::time::Duration::from_millis(100));
        unsafe {
            esp_idf_svc::sys::esp_restart();
        }
    }));
}
