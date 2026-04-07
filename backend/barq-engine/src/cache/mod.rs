//! Cache layer for CDN functionality

use anyhow::Result;
use bytes::Bytes;
use parking_lot::RwLock;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use std::sync::Arc;
use std::time::Duration;
use lru::LruCache;
use std::num::NonZeroUsize;
use tracing::info;

/// Multi-tier cache: Memory (L1) + Redis (L2)
pub struct CacheLayer {
    /// L1: In-memory LRU cache (hot objects)
    memory_cache: Arc<RwLock<LruCache<String, Bytes>>>,
    /// L2: Redis distributed cache
    redis: ConnectionManager,
    /// Default TTL for cached objects
    default_ttl: Duration,
}

impl CacheLayer {
    /// Create new cache layer
    pub async fn new(redis_url: &str, memory_size: usize, default_ttl: Duration) -> Result<Self> {
        info!("🗄️  Initializing cache layer...");
        info!("   Memory cache size: {} objects", memory_size);
        info!("   Redis URL: {}", redis_url);
        
        let client = Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;
        
        // Test Redis connection
        let mut conn = redis.clone();
        let _: () = redis::cmd("PING").query_async(&mut conn).await?;
        
        let memory_cache = Arc::new(RwLock::new(
            LruCache::new(NonZeroUsize::new(memory_size).unwrap())
        ));

        info!("✅ Cache layer initialized");

        Ok(Self {
            memory_cache,
            redis,
            default_ttl,
        })
    }

    /// Get object from cache (L1 -> L2 -> miss)
    pub async fn get(&self, key: &str) -> Option<Bytes> {
        // Try L1 (memory)
        if let Some(data) = self.memory_cache.write().get(key).cloned() {
            info!("🎯 L1 cache HIT: {}", key);
            return Some(data);
        }

        // Try L2 (Redis)
        let redis_key = format!("barq:obj:{}", key);
        let mut conn = self.redis.clone();
        
        match conn.get::<_, Vec<u8>>(&redis_key).await {
            Ok(data) => {
                info!("🎯 L2 cache HIT: {}", key);
                let bytes = Bytes::from(data);
                
                // Promote to L1
                self.memory_cache.write().put(key.to_string(), bytes.clone());
                
                Some(bytes)
            }
            Err(_) => {
                info!("❌ Cache MISS: {}", key);
                None
            }
        }
    }

    /// Put object in cache (L1 + L2)
    pub async fn put(&self, key: &str, data: Bytes, ttl: Option<Duration>) -> Result<()> {
        let ttl = ttl.unwrap_or(self.default_ttl);
        
        // Store in L1 (memory)
        self.memory_cache.write().put(key.to_string(), data.clone());

        // Store in L2 (Redis)
        let redis_key = format!("barq:obj:{}", key);
        let mut conn = self.redis.clone();
        
        let _: () = conn.set_ex(&redis_key, data.to_vec(), ttl.as_secs()).await?;
        
        info!("💾 Cached: {} (TTL: {}s)", key, ttl.as_secs());
        
        Ok(())
    }

    /// Invalidate cached object
    pub async fn invalidate(&self, key: &str) -> Result<()> {
        // Remove from L1
        self.memory_cache.write().pop(key);

        // Remove from L2
        let redis_key = format!("barq:obj:{}", key);
        let mut conn = self.redis.clone();
        
        let _: () = conn.del(&redis_key).await?;
        
        info!("🗑️  Invalidated: {}", key);
        
        Ok(())
    }

    /// Clear all caches
    pub async fn clear(&self) -> Result<()> {
        // Clear L1
        self.memory_cache.write().clear();

        // Clear L2 (all barq:obj:* keys)
        let mut conn = self.redis.clone();
        
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg("barq:obj:*")
            .query_async(&mut conn)
            .await?;

        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }

        info!("🧹 Cache cleared");
        
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let l1_size = self.memory_cache.read().len();
        
        let mut conn = self.redis.clone();
        let l2_size: usize = redis::cmd("DBSIZE")
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        CacheStats {
            l1_size,
            l2_size,
            l1_capacity: self.memory_cache.read().cap().get(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l2_size: usize,
    pub l1_capacity: usize,
}
