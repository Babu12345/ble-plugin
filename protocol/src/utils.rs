//! Common utils for the protocol

/// Convert a slice to an array of fixed size
pub fn slice_to_array<const N: usize>(slice: &[u8]) -> crate::errors::Result<[u8; N]> {
    <[u8; N]>::try_from(slice).map_err(|_| {
        return crate::errors::Error::InvalidDataLength {
            expected: N,
            got: slice.len(),
        };
    })
}
