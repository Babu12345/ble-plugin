//! Traits that can be used by specific devices for communications

/// Read throttling information
pub struct ReadThrottleInfo {
    /// Throttle timeout
    pub timeout: core::time::Duration,
    /// Throttle timeout threshold
    pub threshold_for_timeout: usize,
}

/// Write throttling information
pub struct WriteThrottleInfo {
    /// Write delay
    pub delay: core::time::Duration,
}

impl WriteThrottleInfo {
    /// Set the the write delay
    pub fn set_delay(mut self, delay: core::time::Duration) -> Self {
        self.delay = delay;
        self
    }
}

impl Default for ReadThrottleInfo {
    fn default() -> Self {
        Self {
            timeout: core::time::Duration::from_millis(10),
            threshold_for_timeout: 10,
        }
    }
}

impl Default for WriteThrottleInfo {
    fn default() -> Self {
        Self {
            delay: core::time::Duration::from_millis(5),
        }
    }
}

/// Plugin device communication
pub mod plugin {
    use core::future::Future;
    #[cfg(feature = "std")]
    use std::thread::Scope;

    use embassy_sync::blocking_mutex::raw::RawMutex;

    use crate::plugin::{AsyncPluginReceiver, AsyncPluginSender};
    #[cfg(feature = "std")]
    use crate::{
        devices::{ReadThrottleInfo, WriteThrottleInfo},
        plugin::plugin::{PluginReceiver, PluginSender},
    };

    /// Trait for processing data in and out of the device
    #[cfg(feature = "std")]
    pub trait PluginProcessor<const CH_SIZE: usize, ERROR> {
        /// Handles the generation of the processors for receiving and sending data
        fn processors<'a, 'b>(
            self,
            scope: &'a Scope<'a, 'b>,
            channel_buffer_size: usize,
            read_throttle_info: ReadThrottleInfo,
            write_throttle_info: WriteThrottleInfo,
        ) -> Result<(PluginSender<CH_SIZE>, PluginReceiver<CH_SIZE>), ERROR>;
    }

    /// Async host processor
    pub trait AsyncPluginProcessor<
        const CH_SIZE: usize,
        const BUFFER_SIZE: usize,
        R: RawMutex,
        ERROR,
    >
    {
        /// To and from definitions
        type T<'ch>
        where
            R: 'ch;

        /// Handles the generation of the processors for receiving and sending data
        fn processors<'ch>(
            self,
            to: Self::T<'ch>,
            from: Self::T<'ch>,
        ) -> Result<
            (
                impl Future<Output = ()>,
                AsyncPluginSender<'ch, R, BUFFER_SIZE, CH_SIZE>,
                AsyncPluginReceiver<'ch, R, BUFFER_SIZE, CH_SIZE>,
            ),
            ERROR,
        >
        where
            R: 'ch;
    }
}

/// Host device communication
pub mod host {
    use core::future::Future;
    #[cfg(feature = "std")]
    use std::thread::Scope;

    use embassy_sync::blocking_mutex::raw::RawMutex;

    use crate::host::{AsyncHostReceiver, AsyncHostSender};
    #[cfg(feature = "std")]
    use crate::{
        devices::{ReadThrottleInfo, WriteThrottleInfo},
        host::{HostReceiver, HostSender},
    };

    /// Trait for processing data in and out of the host device
    #[cfg(feature = "std")]
    pub trait HostProcessor<const CH_SIZE: usize, ERROR> {
        /// Handles the generation of the processors for receiving and sending data
        fn processors<'a, 'b>(
            self,
            scope: &'a Scope<'a, 'b>,
            channel_buffer_size: usize,
            read_throttle_info: ReadThrottleInfo,
            write_throttle_info: WriteThrottleInfo,
        ) -> Result<(HostSender<CH_SIZE>, HostReceiver<CH_SIZE>), ERROR>;
    }

    /// Async host processor
    pub trait AsyncHostProcessor<const CH_SIZE: usize, const BUFFER_SIZE: usize, R: RawMutex, ERROR>
    {
        /// To and from definitions
        type T<'ch>
        where
            R: 'ch;

        /// Handles the generation of the processors for receiving and sending data
        fn processors<'ch>(
            self,
            to: Self::T<'ch>,
            from: Self::T<'ch>,
        ) -> Result<
            (
                impl Future<Output = ()>,
                AsyncHostSender<'ch, R, BUFFER_SIZE, CH_SIZE>,
                AsyncHostReceiver<'ch, R, BUFFER_SIZE, CH_SIZE>,
            ),
            ERROR,
        >
        where
            R: 'ch;
    }
}
