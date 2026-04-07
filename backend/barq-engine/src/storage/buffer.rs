//! Aligned buffer management for zero-copy DMA operations
//!
//! NVMe drives require buffers to be aligned to sector boundaries (typically 4KB)
//! for direct I/O. This module provides safe abstractions for aligned memory.

use super::{Result, StorageError};
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::slice;

/// A buffer aligned to hardware sector boundaries for zero-copy I/O
#[allow(dead_code)]
pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    layout: Layout,
    len: usize,
    alignment: usize,
}

#[allow(dead_code)]
impl AlignedBuffer {
    /// Creates a new aligned buffer with the specified size and alignment
    ///
    /// # Arguments
    /// * `size` - Buffer size in bytes
    /// * `alignment` - Alignment in bytes (typically 4096 for NVMe)
    ///
    /// # Safety
    /// The alignment must be a power of 2
    pub fn new(size: usize, alignment: usize) -> Result<Self> {
        if !alignment.is_power_of_two() {
            return Err(StorageError::AlignmentError {
                required: alignment,
                actual: 0,
            });
        }

        // Align size up to the next multiple of alignment
        let aligned_size = (size + alignment - 1) & !(alignment - 1);

        let layout = Layout::from_size_align(aligned_size, alignment)
            .map_err(|_| StorageError::AlignmentError {
                required: alignment,
                actual: 0,
            })?;

        // SAFETY: Layout is valid, we check for null below
        let ptr = unsafe { alloc(layout) };

        let ptr = NonNull::new(ptr).ok_or_else(|| StorageError::IoError(
            "Memory allocation failed".to_string()
        ))?;

        Ok(Self {
            ptr,
            layout,
            len: aligned_size,
            alignment,
        })
    }

    /// Returns the buffer as a mutable byte slice
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: ptr is valid and aligned, len is correct
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    /// Returns the buffer as a byte slice
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is valid and aligned, len is correct
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Returns the raw pointer for FFI operations
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Returns the mutable raw pointer for FFI operations
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Returns the buffer length
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the buffer alignment
    #[inline]
    pub fn alignment(&self) -> usize {
        self.alignment
    }
}

// SAFETY: AlignedBuffer owns its memory and is safe to send across threads
unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        // SAFETY: ptr and layout are valid
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_buffer() {
        let mut buf = AlignedBuffer::new(8192, 4096).unwrap();
        assert_eq!(buf.len(), 8192);
        assert_eq!(buf.alignment(), 4096);
        assert_eq!(buf.as_ptr() as usize % 4096, 0);
    }

    #[test]
    fn test_buffer_write_read() {
        let mut buf = AlignedBuffer::new(4096, 4096).unwrap();
        let slice = buf.as_mut_slice();
        slice[0] = 42;
        slice[100] = 255;
        
        assert_eq!(buf.as_slice()[0], 42);
        assert_eq!(buf.as_slice()[100], 255);
    }
}
