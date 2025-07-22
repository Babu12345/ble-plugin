//! Plugin device. In this case it will be bluetooth in order to show the protocol plugin implementation
#![deny(missing_docs)]

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

#[macro_export]
/// Concat N arrays
macro_rules! concat_n_arrays {
    ($arr:expr) => { $arr };

    ($arr1:expr, $arr2:expr) => {{
        let a1 = $arr1;
        let a2 = $arr2;
        std::array::from_fn(|i| {
            if i < a1.len() { a1[i] } else { a2[i - a1.len()] }
        })
    }};

    ($first:expr, $($rest:expr),+) => {
        concat_arrays!(
            $first,
            concat_arrays!($($rest),+)
        )
    };
}
