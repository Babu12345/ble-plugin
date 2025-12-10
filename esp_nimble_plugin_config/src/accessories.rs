// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Hardware accessors for the Esp32

use plugin_config::HardwareAccessories;
use std::sync::Mutex;
use std::time::Duration;
use threadpool::ThreadPool;
use throttle::Throttle;

use esp_idf_svc::hal::gpio::AnyOutputPin;
use esp_idf_svc::hal::gpio::Output;
use esp_idf_svc::hal::gpio::PinDriver;
use std::sync::Arc;

/// Esp32's hardware accessories
pub struct EspHardwareAccessories {
    /// Pin indicator
    indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>,
    /// Throttle for blink indication to prevent excessive blinking
    /// and errors
    blink_throttle: Throttle,
    /// Thread pool for managing blink operations
    blink_thread_pool: ThreadPool,
}

impl EspHardwareAccessories {
    /// New instance
    pub fn new(indicator: Arc<Mutex<PinDriver<'static, AnyOutputPin, Output>>>) -> Self {
        Self {
            indicator,
            blink_throttle: Throttle::new(Duration::from_millis(500), 1),
            blink_thread_pool: ThreadPool::new(1),
        }
    }
}

impl HardwareAccessories for EspHardwareAccessories {
    fn blink(&mut self, state: plugin_config::BlinkState) {
        // Apply throttling
        match self.blink_throttle.accept() {
            Ok(_) => {}
            Err(_) => {
                log::debug!("Blink indication throttled");
                return;
            }
        }

        let indicator = self.indicator.clone();

        // Submit blink task to thread pool
        self.blink_thread_pool.execute(move || {
            for i in 0..4 {
                // Try to acquire lock non-blocking
                match indicator.try_lock() {
                    Ok(mut indicator) => {
                        if let Err(e) = {
                            match i % 2 {
                                0 => indicator.set_low(),
                                _ => indicator.set_high(),
                            }
                        } {
                            log::error!("Failed to toggle GPIO: {:?}", e);
                            return;
                        }
                    }
                    Err(_) => {
                        log::debug!("GPIO lock busy, skipping blink");
                        return;
                    }
                }

                // Sleep after releasing the lock
                match state {
                    plugin_config::BlinkState::Success => {
                        std::thread::sleep(Duration::from_millis(if i == 0 { 50 } else { 5 }));
                    }
                    plugin_config::BlinkState::Failure => {
                        std::thread::sleep(Duration::from_millis(40));
                    }
                }
            }
        });
    }
}
