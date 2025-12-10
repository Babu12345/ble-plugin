// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Util types and functions

use esp_idf_sys::cherry_host::usbh_cdc_acm;
use protocol::DEFAULT_PACKET_SIZE;

#[derive(Debug)]
pub(crate) struct ThreadSafeCDCWrapper(pub *mut usbh_cdc_acm);
unsafe impl Send for ThreadSafeCDCWrapper {}
unsafe impl Sync for ThreadSafeCDCWrapper {}

pub type TSenderAndReceiver = [u8; DEFAULT_PACKET_SIZE];
