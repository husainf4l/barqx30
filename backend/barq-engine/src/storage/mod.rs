//! High-performance storage engine with io_uring and zero-copy I/O
//!
//! This module implements the core storage layer of BARQ X30, targeting
//! 30-microsecond latency through:
//! - io_uring for async kernel I/O (Linux)
//! - tokio::fs for development (macOS)
//! - Aligned buffers for zero-copy DMA
//! - CPU affinity for deterministic performance

mod engine;
mod buffer;

// Direct I/O only on Linux
#[cfg(target_os = "linux")]
mod io;

pub use engine::StorageEngine;

#[cfg(target_os = "linux")]
pub use buffer::AlignedBuffer;

#[cfg(target_os = "linux")]
pub use io::DirectIO;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("I/O operation failed: {0}")]
    IoError(String),
    
    #[error("Buffer alignment error: required {required}, got {actual}")]
    AlignmentError { required: usize, actual: usize },
    
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    
    #[error("Storage quota exceeded")]
    QuotaExceeded,
    
    #[error("Invalid object key: {0}")]
    InvalidKey(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
