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
