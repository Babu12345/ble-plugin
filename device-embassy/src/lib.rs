// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Library for the usb device embassy implementation

#![no_std]
#![deny(missing_docs)]
#![cfg(all(target_arch = "xtensa", target_os = "none"))]

pub mod errors;
pub mod processors;
