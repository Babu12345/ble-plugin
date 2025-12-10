// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! USB host implementation library of cherry usb
//!
//! Uses `heapless` for internal buffers. Protocol types use `alloc::Vec` and
//! `alloc::String` for Protocol Buffer compatibility.

#![cfg(all(target_arch = "xtensa", target_os = "espidf"))]
#[deny(missing_docs)]
mod processors;
mod utils;

pub use processors::*;

// Initialization examples
// https://github.com/zleihao/CherryUSB-CDC-MSC/blob/50095e0b63bbdf6f2d5597e71edfa45dd8be6c1d/cdc_msc/middlewares/CherryUSB-1.4.0/class/cdc/usbh_cdc_acm.c#L170
// https://github.com/cherry-embedded/CherryUSB/blob/f23f5494920b64987350abc87c8154f410c6f5f9/platform/nuttx/usbh_serial.c#L180
// https://github.com/search?q=repo%3Acherry-embedded%2FCherryUSB%20usbh_cdc_acm_run&type=code
// https://github.com/hpmicro/zephyr_sdk_glue/blob/2a17ddea9f43eac3b7f57a0058ce49023d5fd06f/samples/cherryusb/host/cdc_acm/src/cdc_acm_chost.c#L33
// https://github.com/CherryUSB/cherryusb_esp32/blob/main/examples/host/sdkconfig
