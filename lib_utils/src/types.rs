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

    /// Get the raw data
    pub fn get_data(&self) -> [u8; N] {
        self.data
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
