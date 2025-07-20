//! Plugin device. In this case it will be bluetooth in order to show the protocol plugin implementation
#![deny(missing_docs)]

pub mod usb_device;

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

// 4-byte aligned buffer wrapper
#[repr(align(4))]
struct AlignedBuffer {
    data: [u8; 2048],
}

impl AlignedBuffer {
    const fn new() -> Self {
        Self { data: [0; 2048] }
    }

    fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl std::ops::Index<usize> for AlignedBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for AlignedBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for AlignedBuffer {
    type Output = [u8];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<std::ops::Range<usize>> for AlignedBuffer {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.data[index]
    }
}
