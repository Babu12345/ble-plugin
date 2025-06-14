//! Library to compartimentalize the host controls and only return the channels.
#[deny(missing_docs)]
mod consts;
mod processors;
use processors::{FROM_USB_SENDER, T, process_usb_cdc_host, start_usb_host};

use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread::Scope,
};

pub struct IO {
    pub sender: SyncSender<T>,
    pub receiver: Receiver<T>,
}

/// Starts the usb host processors and returns channels to communiate with the device
pub unsafe fn usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>, bound: usize) -> IO {
    let to_usb = mpsc::sync_channel(bound);
    let from_usb = {
        let channel = mpsc::sync_channel(bound);
        FROM_USB_SENDER.set(channel.0).unwrap();
        channel.1
    };

    unsafe {
        start_usb_host(scope);
        scope.spawn(move || process_usb_cdc_host(to_usb.1));
    }

    IO {
        sender: to_usb.0,
        receiver: from_usb,
    }
}
