//! Traits that can be used by specific devices for communications

/// Plugin device communication
pub mod plugin {
    use core::future::Future;
    #[cfg(feature = "std")]
    use core::time::Duration;
    #[cfg(feature = "std")]
    use std::thread::Scope;

    use embassy_sync::blocking_mutex::raw::RawMutex;

    #[cfg(feature = "std")]
    use crate::plugin::plugin::{PluginReceiver, PluginSender};
    use crate::plugin::{AsyncPluginReceiver, AsyncPluginSender};

    /// Trait for processing data in and out of the device
    #[cfg(feature = "std")]
    pub trait PluginProcessor<const CH_SIZE: usize, ERROR> {
        /// Handles the generation of the processors for receiving and sending data
        fn processors<'a, 'b>(
            self,
            scope: &'a Scope<'a, 'b>,
            channel_buffer_size: usize,
            throttle_info: (Duration, usize),
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
    use core::time::Duration;
    #[cfg(feature = "std")]
    use std::thread::Scope;

    use embassy_sync::blocking_mutex::raw::RawMutex;

    use crate::host::{AsyncHostReceiver, AsyncHostSender};
    #[cfg(feature = "std")]
    use crate::host::{HostReceiver, HostSender};

    /// Trait for processing data in and out of the device
    #[cfg(feature = "std")]
    pub trait HostProcessor<const CH_SIZE: usize, ERROR> {
        /// Handles the generation of the processors for receiving and sending data
        fn processors<'a, 'b>(
            self,
            scope: &'a Scope<'a, 'b>,
            channel_buffer_size: usize,
            throttle_info: (Duration, usize),
        ) -> Result<(HostSender<CH_SIZE>, HostReceiver<CH_SIZE>), ERROR>;
    }

    /// Async host processor
    pub trait AsyncHostProcessor<const CH_SIZE: usize, const BUFFER_SIZE: usize, R: RawMutex, ERROR>
    {
        /// To and from definitions
        type T<'ch>
        where
            R: 'ch;
        // type T<'ch, A, B, const C: usize>;
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
