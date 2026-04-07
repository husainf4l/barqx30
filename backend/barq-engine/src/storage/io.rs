//! Direct I/O operations bypassing OS page cache
//!
//! This module provides O_DIRECT file operations for predictable,
//! low-latency disk access.

use super::{AlignedBuffer, Result, StorageError};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use tracing::{debug, trace};

/// Direct I/O file handle with O_DIRECT flag
pub struct DirectIO {
    file: File,
    alignment: usize,
}

impl DirectIO {
    /// Opens a file with O_DIRECT flag for zero-copy I/O
    ///
    /// # Arguments
    /// * `path` - File path
    /// * `alignment` - Buffer alignment requirement (typically 4096)
    pub fn open<P: AsRef<Path>>(path: P, alignment: usize) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .custom_flags(libc::O_DIRECT) // Bypass kernel page cache
            .open(path.as_ref())
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        debug!("Opened file with O_DIRECT: {:?}", path.as_ref());

        Ok(Self { file, alignment })
    }

    /// Reads data into an aligned buffer
    ///
    /// # Arguments
    /// * `offset` - File offset (must be aligned)
    /// * `buffer` - Aligned buffer to read into
    ///
    /// # Returns
    /// Number of bytes read
    pub fn read_at(&self, offset: u64, buffer: &mut AlignedBuffer) -> Result<usize> {
        // Verify alignment
        if offset as usize % self.alignment != 0 {
            return Err(StorageError::AlignmentError {
                required: self.alignment,
                actual: offset as usize % self.alignment,
            });
        }

        use std::os::unix::fs::FileExt;
        let bytes_read = self.file.read_at(buffer.as_mut_slice(), offset)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        trace!("Read {} bytes at offset {}", bytes_read, offset);
        Ok(bytes_read)
    }

    /// Writes data from an aligned buffer
    ///
    /// # Arguments
    /// * `offset` - File offset (must be aligned)
    /// * `buffer` - Aligned buffer to write from
    ///
    /// # Returns
    /// Number of bytes written
    pub fn write_at(&self, offset: u64, buffer: &AlignedBuffer) -> Result<usize> {
        // Verify alignment
        if offset as usize % self.alignment != 0 {
            return Err(StorageError::AlignmentError {
                required: self.alignment,
                actual: offset as usize % self.alignment,
            });
        }

        use std::os::unix::fs::FileExt;
        let bytes_written = self.file.write_at(buffer.as_slice(), offset)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        trace!("Wrote {} bytes at offset {}", bytes_written, offset);
        Ok(bytes_written)
    }

    /// Synchronizes file data to disk (fsync)
    pub fn sync(&self) -> Result<()> {
        self.file.sync_data()
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        debug!("Synchronized file to disk");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_direct_io_write_read() {
        let path = "/tmp/barq_test_direct_io";
        
        // Write
        {
            let mut buf = AlignedBuffer::new(4096, 4096).unwrap();
            buf.as_mut_slice()[0] = 42;
            buf.as_mut_slice()[100] = 255;
            
            let dio = DirectIO::open(path, 4096).unwrap();
            let written = dio.write_at(0, &buf).unwrap();
            assert_eq!(written, 4096);
            dio.sync().unwrap();
        }
        
        // Read
        {
            let mut buf = AlignedBuffer::new(4096, 4096).unwrap();
            let dio = DirectIO::open(path, 4096).unwrap();
            let read = dio.read_at(0, &mut buf).unwrap();
            assert_eq!(read, 4096);
            assert_eq!(buf.as_slice()[0], 42);
            assert_eq!(buf.as_slice()[100], 255);
        }
        
        fs::remove_file(path).ok();
    }
}
