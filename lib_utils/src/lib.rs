#![no_std]
#![deny(missing_docs)]
//! Common utilities for the workspace

/// Custom slice trait for trimming or extending slices
pub trait MatchSliceLengths<const N: usize> {
    /// Match size of the output
    fn match_size(self, padding: u8) -> [u8; N];
}

impl<const N: usize> MatchSliceLengths<N> for &[u8] {
    #[inline(always)]
    fn match_size(self, padding: u8) -> [u8; N] {
        let mut buffer = [padding; N];
        let array_size = self.len();
        if N >= array_size {
            buffer[..array_size].copy_from_slice(self);
            return buffer;
        }
        buffer.copy_from_slice(&self[..N]);
        buffer
    }
}

#[macro_export]
/// Makes an object static even after the start of the program.
/// When you are okay with using a nightly compiler it's better to use [make_static](https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html)
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[macro_export]
/// Macro to ensure exactly one feature from a group is enabled
macro_rules! exactly_one_feature {
    ($($feature:literal),+) => {
        const _: () = {
            let mut count = 0;
            $(
                if cfg!(feature = $feature) { count += 1; }
            )+
            assert!(count == 1, "Exactly one feature must be enabled from the list");
        };
    };
}

#[cfg(test)]
mod tests {
    use crate::MatchSliceLengths;

    #[test]
    fn test_match_size() {
        let buffer = &[1u8, 2u8, 3u8];

        let fixed_buffer: [u8; 5] = buffer.match_size(0x00);
        assert_eq!(
            fixed_buffer,
            [1u8, 2u8, 3u8, 0u8, 0u8],
            "Fixed buffer should equal the normal buffer except for some 0 padding"
        );

        let fixed_buffer: [u8; 5] = buffer.match_size(0xff);
        assert_eq!(
            fixed_buffer,
            [1u8, 2u8, 3u8, 0xff, 0xff],
            "Fixed buffer should equal the normal buffer except for some 0xff padding"
        );

        let fixed_buffer: [u8; 2] = buffer.match_size(0xff);
        assert_eq!(
            fixed_buffer,
            [1u8, 2u8],
            "Fixed buffer should reduce the size of the normal buffer"
        );
    }
}
