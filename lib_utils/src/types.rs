//! Creation of new custom types

/// 4-byte aligned buffer wrapper
#[repr(C, align(4))]
pub struct AlignedBuffer<const N: usize> {
    data: [u8; N],
}

impl<const N: usize> AlignedBuffer<N> {
    /// Construct a new AlignedBuffer
    pub const fn new() -> Self {
        Self { data: [0; N] }
    }

    /// Covert to a mutable pointer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    /// Covert to a pointer
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Get the raw data
    pub fn get_data(&self) -> [u8; N] {
        self.data
    }

    /// Get a mutable reference
    pub fn as_mut<'a>(&'a mut self) -> &'a mut [u8; N] {
        &mut self.data
    }

    /// Get the raw data
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<const N: usize> core::ops::Index<usize> for AlignedBuffer<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> core::ops::IndexMut<usize> for AlignedBuffer<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize> core::ops::Index<core::ops::Range<usize>> for AlignedBuffer<N> {
    type Output = [u8];

    fn index(&self, index: core::ops::Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> core::ops::IndexMut<core::ops::Range<usize>> for AlignedBuffer<N> {
    fn index_mut(&mut self, index: core::ops::Range<usize>) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize> From<[u8; N]> for AlignedBuffer<N> {
    fn from(data: [u8; N]) -> Self {
        Self { data }
    }
}

impl<const N: usize> Into<[u8; N]> for AlignedBuffer<N> {
    fn into(self) -> [u8; N] {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buffer_alignment() {
        let buffer = AlignedBuffer::<64>::new();
        let ptr = buffer.as_ptr() as usize;
        assert_eq!(ptr % 4, 0, "AlignedBuffer should be 4-byte aligned");
    }

    #[test]
    fn test_aligned_buffer_from_into() {
        let data = [1, 2, 3, 4];
        let buffer = AlignedBuffer::from(data);
        assert_eq!(buffer.get_data(), data);

        // Test that buffer created from data is also 4-byte aligned
        let ptr = buffer.as_ptr() as usize;
        assert_eq!(
            ptr % 4,
            0,
            "AlignedBuffer created from data should be 4-byte aligned"
        );

        let retrieved: [u8; 4] = buffer.into();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_aligned_buffer_indexing() {
        let mut buffer = AlignedBuffer::from([10, 20, 30, 40]);
        assert_eq!(buffer[0], 10);
        assert_eq!(buffer[3], 40);

        buffer[1] = 25;
        assert_eq!(buffer[1], 25);

        assert_eq!(&buffer[1..3], &[25, 30]);
    }
}
