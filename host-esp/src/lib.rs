// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

//! Library to compartimentalize the host controls and only return the channels. Uses the standard espressif esp libary
#[deny(missing_docs)]
mod constants;
mod processors;
use processors::{FROM_USB_SENDER, T, process_usb_cdc_host, start_usb_host};
use protocol::host::{HostReceiver, HostSender};

use protocol::DEFAULT_PACKET_SIZE;
use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread::Scope,
};
pub struct IO {
    pub sender: SyncSender<T>,
    pub receiver: Receiver<T>,
}

/// Starts the usb host processors and returns channels to communiate with the device
pub unsafe fn usb_host<'a, 'b>(
    scope: &'a Scope<'a, 'b>,
    channel_buffer_size: usize,
) -> (
    HostSender<DEFAULT_PACKET_SIZE>,
    HostReceiver<DEFAULT_PACKET_SIZE>,
) {
    let to_usb = mpsc::sync_channel(channel_buffer_size);
    let from_usb = {
        let channel = mpsc::sync_channel(channel_buffer_size);
        FROM_USB_SENDER.set(channel.0).unwrap();
        channel.1
    };

    unsafe {
        start_usb_host(scope);
        scope.spawn(move || process_usb_cdc_host(to_usb.1));
    }

    (HostSender::new(to_usb.0), HostReceiver::new(from_usb))
}
