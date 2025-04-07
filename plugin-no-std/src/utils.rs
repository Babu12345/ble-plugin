//! Common util functions for the plugin-no-std module

use core::future;

#[macro_export]
/// Makes an object static even after the start of the program.
/// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// Helper function to indefinitely await.
pub async fn indefinitely() {
    future::pending::<()>().await;
}
