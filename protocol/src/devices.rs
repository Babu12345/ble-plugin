//! Traits that can be used by specific devices for communications

/// Plugin device communication
pub mod plugin {
    #[cfg(feature = "std")]
    use core::time::Duration;
    #[cfg(feature = "std")]
    use std::thread::Scope;

    #[cfg(feature = "std")]
    use crate::plugin::plugin::{PluginReceiver, PluginSender};

    /// Trait for processing data in and out of the device
    #[cfg(feature = "std")]
    pub trait PluginProcessor<const SIZE: usize, ERROR> {
        /// Handles the generation of the processors for receiving and sending data
        fn processors<'a, 'b>(
            self,
            scope: &'a Scope<'a, 'b>,
            channel_buffer_size: usize,
            throttle_info: (Duration, usize),
        ) -> Result<(PluginSender<SIZE>, PluginReceiver<SIZE>), ERROR>;
    }
}
