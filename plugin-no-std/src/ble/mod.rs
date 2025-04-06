//! BLE processor and runner

use trouble_host::{Controller, prelude::Runner};

use crate::configs::Server;

/// Run the Bluetooth peripheral
pub async fn run<'runner, C>(mut runner: Runner<'runner, C>)
where
    C: Controller,
{
    loop {
        runner
            .run()
            .await
            .inspect_err(|_| log::error!("BLE runner error occurred"))
            .ok();
    }
}

/// Run the Bluetooth peripheral
pub async fn processor(_server: Server<'_>) {}
