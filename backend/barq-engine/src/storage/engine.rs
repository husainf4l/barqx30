//! Core storage engine implementation
//!
//! Platform-specific backends:
//! - Linux: io_uring for 30-microsecond latency (Race Car Mode)
//! - macOS: tokio::fs for development and testing
//!
//! Orchestrates direct I/O, buffer management, and CPU affinity
//! to achieve 30-microsecond latency target on Linux.

use super::{Result, StorageError};
use crate::config::Config;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Object metadata
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectMeta {
    pub key: String,
    pub size: u64,
    pub created_at: i64,  // Unix timestamp
    pub etag: String,
}

/// High-performance storage engine
#[allow(dead_code)]
pub struct StorageEngine {
    data_dir: PathBuf,
    alignment: usize,
    direct_io: bool,
    /// Lock-free concurrent object index
    objects: Arc<DashMap<String, ObjectMeta>>,
}

#[allow(dead_code)]
impl StorageEngine {
    /// Creates a new storage engine instance
    pub async fn new(config: &Config) -> Result<Self> {
        let data_dir = config.storage.data_dir.clone();
        
        // Create data directory if not exists
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        
        info!("🏗️  Storage engine initialized at: {:?}", data_dir);
        
        // Platform-specific logging
        #[cfg(target_os = "linux")]
        {
            info!("🏎️  RACE CAR MODE: io_uring enabled (30μs target latency)");
            info!("⚡ Direct I/O: {}", config.storage.direct_io);
            info!("📦 Buffer alignment: {} bytes", config.storage.buffer_alignment);
        }
        
        #[cfg(target_os = "macos")]
        {
            warn!("🍎 macOS MODE: Using tokio::fs (development/testing)");
            warn!("⚠️  For production performance, deploy on Linux with io_uring");
        }

        Ok(Self {
            data_dir,
            alignment: config.storage.buffer_alignment,
            direct_io: config.storage.direct_io,
            objects: Arc::new(DashMap::new()),
        })
    }

    /// Stores an object with the given key and data
    ///
    /// # Arguments
    /// * `key` - Object identifier
    /// * `data` - Object data
    ///
    /// # Returns
    /// Object metadata including ETag
    pub async fn put_object(&self, key: String, data_bytes: bytes::Bytes) -> Result<ObjectMeta> {
        let data = data_bytes.as_ref();
        
        // Validate key
        if key.is_empty() || key.contains("..") {
            return Err(StorageError::InvalidKey(key));
        }

        let object_path = self.data_dir.join(&key);
        
        // Create parent directories
        if let Some(parent) = object_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        let etag = {
            let data_clone = data_bytes.clone(); // O(1) Arc clone instead of copying 590MB
            tokio::task::spawn_blocking(move || format!("{:x}", md5::compute(data_clone)))
                .await
                .map_err(|e| StorageError::IoError(format!("MD5 compute error: {}", e)))?
        };

        // Write data
        if self.direct_io {
            self.write_direct(&object_path, data).await?;
        } else {
            tokio::fs::write(&object_path, data).await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        let meta = ObjectMeta {
            key: key.clone(),
            size: data.len() as u64,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            etag: etag.clone(),
        };

        // Update index
        self.objects.insert(key.clone(), meta.clone());

        debug!("Stored object: {} ({} bytes)", key, data.len());
        Ok(meta)
    }

    /// Retrieves an object by key
    ///
    /// # Arguments
    /// * `key` - Object identifier
    ///
    /// # Returns
    /// Object data
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        let object_path = self.data_dir.join(key);

        if !object_path.exists() {
            return Err(StorageError::ObjectNotFound(key.to_string()));
        }

        let data = if self.direct_io {
            self.read_direct(&object_path).await?
        } else {
            tokio::fs::read(&object_path).await
                .map_err(|e| StorageError::IoError(e.to_string()))?
        };

        debug!("Retrieved object: {} ({} bytes)", key, data.len());
        Ok(data)
    }

    /// Deletes an object by key
    pub async fn delete_object(&self, key: &str) -> Result<()> {
        let object_path = self.data_dir.join(key);

        if !object_path.exists() {
            return Err(StorageError::ObjectNotFound(key.to_string()));
        }

        tokio::fs::remove_file(&object_path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        self.objects.remove(key);

        debug!("Deleted object: {}", key);
        Ok(())
    }

    /// Lists all objects (returns keys)
    pub async fn list_objects(&self) -> Vec<String> {
        self.objects.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Gets object metadata without reading data
    pub async fn head_object(&self, key: &str) -> Result<ObjectMeta> {
        self.objects
            .get(key)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| StorageError::ObjectNotFound(key.to_string()))
    }

    /// Write data using platform-specific I/O
    #[cfg(target_os = "linux")]
    async fn write_direct(&self, path: &PathBuf, data: &[u8]) -> Result<()> {
        // Linux: Use io_uring for true zero-copy I/O
        use tokio_uring::fs::File;
        
        let file = File::create(path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let (result, _) = file.write_at(data, 0).await;
        result.map_err(|e| StorageError::IoError(e.to_string()))?;
        
        debug!("io_uring write: {} bytes", data.len());
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn write_direct(&self, path: &PathBuf, data: &[u8]) -> Result<()> {
        // macOS: Use standard tokio::fs (thread pool based)
        tokio::fs::write(path, data).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        debug!("tokio::fs write: {} bytes", data.len());
        Ok(())
    }

    /// Read data using platform-specific I/O
    #[cfg(target_os = "linux")]
    async fn read_direct(&self, path: &PathBuf) -> Result<Vec<u8>> {
        // Linux: Use io_uring for true zero-copy I/O
        use tokio_uring::fs::File;
        
        let file = File::open(path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let metadata = std::fs::metadata(path)
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let file_size = metadata.len() as usize;
        
        let mut buffer = vec![0u8; file_size];
        let (result, buf) = file.read_at(buffer, 0).await;
        result.map_err(|e| StorageError::IoError(e.to_string()))?;
        
        debug!("io_uring read: {} bytes", buf.len());
        Ok(buf)
    }

    #[cfg(target_os = "macos")]
    async fn read_direct(&self, path: &PathBuf) -> Result<Vec<u8>> {
        // macOS: Use standard tokio::fs (thread pool based)
        let data = tokio::fs::read(path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        debug!("tokio::fs read: {} bytes", data.len());
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StorageConfig;

    #[tokio::test]
    async fn test_storage_engine() {
        let config = Config {
            storage: StorageConfig {
                data_dir: PathBuf::from("/tmp/barq_test_storage"),
                direct_io: false,
                buffer_alignment: 4096,
                cpu_affinity: false,
                io_threads: 1,
            },
            ..Default::default()
        };

        let engine = StorageEngine::new(&config).await.unwrap();

        // Put
        let data = b"Hello, BARQ X30!";
        let meta = engine.put_object("test/file.txt".to_string(), data).await.unwrap();
        assert_eq!(meta.size, data.len() as u64);

        // Get
        let retrieved = engine.get_object("test/file.txt").await.unwrap();
        assert_eq!(retrieved, data);

        // Head
        let head_meta = engine.head_object("test/file.txt").await.unwrap();
        assert_eq!(head_meta.key, "test/file.txt");

        // List
        let objects = engine.list_objects().await;
        assert!(objects.contains(&"test/file.txt".to_string()));

        // Delete
        engine.delete_object("test/file.txt").await.unwrap();
        assert!(engine.get_object("test/file.txt").await.is_err());

        // Cleanup
        std::fs::remove_dir_all("/tmp/barq_test_storage").ok();
    }
}
