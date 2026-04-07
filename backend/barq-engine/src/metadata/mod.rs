//! LSM-tree based metadata store for lightning-fast file lookups
//!
//! Uses RocksDB/Sled for persistent, high-performance metadata indexing
//! with lock-free concurrent access.

use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::Db;
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub key: String,
    pub size: u64,
    pub chunk_ids: Vec<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub etag: String,
    pub content_type: String,
}

/// High-performance metadata store using LSM-tree
#[allow(dead_code)]
pub struct MetadataStore {
    db: Db,
}

#[allow(dead_code)]
impl MetadataStore {
    /// Creates a new metadata store
    pub fn new(config: &Config) -> Result<Self> {
        let db = sled::open(&config.metadata.db_path)?;
        info!("Metadata store initialized at: {:?}", config.metadata.db_path);
        Ok(Self { db })
    }

    /// Inserts file metadata
    pub fn put(&self, key: &str, metadata: &FileMetadata) -> Result<()> {
        let data = bincode::serialize(metadata)?;
        self.db.insert(key.as_bytes(), data)?;
        Ok(())
    }

    /// Retrieves file metadata
    pub fn get(&self, key: &str) -> Result<Option<FileMetadata>> {
        match self.db.get(key.as_bytes())? {
            Some(data) => {
                let metadata = bincode::deserialize(&data)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    /// Deletes file metadata
    pub fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        Ok(())
    }

    /// Lists all keys with optional prefix
    pub fn list(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        
        match prefix {
            Some(p) => {
                for result in self.db.scan_prefix(p.as_bytes()) {
                    let (key_ivec, _): (sled::IVec, sled::IVec) = result?;
                    keys.push(String::from_utf8_lossy(key_ivec.as_ref()).to_string());
                }
            }
            None => {
                for result in self.db.iter() {
                    let (key_ivec, _): (sled::IVec, sled::IVec) = result?;
                    keys.push(String::from_utf8_lossy(key_ivec.as_ref()).to_string());
                }
            }
        }
        
        Ok(keys)
    }

    /// Flushes pending writes to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetadataConfig;
    use std::path::PathBuf;

    #[test]
    fn test_metadata_store() {
        let config = Config {
            metadata: MetadataConfig {
                db_path: PathBuf::from("/tmp/barq_test_metadata"),
                cache_size: 1024 * 1024,
                use_lsm: true,
            },
            ..Default::default()
        };

        let store = MetadataStore::new(&config).unwrap();

        let metadata = FileMetadata {
            key: "test/file.txt".to_string(),
            size: 1024,
            chunk_ids: vec!["chunk1".to_string(), "chunk2".to_string()],
            created_at: 1234567890,
            modified_at: 1234567890,
            etag: "abc123".to_string(),
            content_type: "text/plain".to_string(),
        };

        // Put
        store.put("test/file.txt", &metadata).unwrap();

        // Get
        let retrieved = store.get("test/file.txt").unwrap().unwrap();
        assert_eq!(retrieved.key, "test/file.txt");
        assert_eq!(retrieved.size, 1024);

        // List
        let keys = store.list(Some("test/")).unwrap();
        assert!(keys.contains(&"test/file.txt".to_string()));

        // Delete
        store.delete("test/file.txt").unwrap();
        assert!(store.get("test/file.txt").unwrap().is_none());

        // Cleanup
        std::fs::remove_dir_all("/tmp/barq_test_metadata").ok();
    }
}
