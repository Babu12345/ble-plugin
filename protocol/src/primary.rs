//! The primary is the data that connects to the plug-in via the host. Typically uses spi / i2c to communicate to
//! the plug-in via the host intermediary. The primary interface is standard enough though that it doesn't need spi/i2c and
//! can directly use USB to transmit to the plug-in if the capability arises.
