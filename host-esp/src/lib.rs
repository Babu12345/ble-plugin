mod consts;
mod processors;
use processors::{FROM_USB_SENDER, T, process_usb_cdc_host, start_usb_host};

use std::{
    sync::mpsc::{self, Receiver, SyncSender},
    thread::Scope,
};

pub struct Out {
    pub to: SyncSender<T>,
    pub from: Receiver<T>,
}

pub unsafe fn usb_host<'a, 'b>(scope: &'a Scope<'a, 'b>, bound: usize) -> Out {
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

    Out {
        to: to_usb.0,
        from: from_usb,
    }
}
