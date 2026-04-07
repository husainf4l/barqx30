//! Cache warming strategies

use crate::cache::CacheLayer;
use anyhow::Result;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

/// Cache warming strategy
pub enum WarmingStrategy {
    /// Warm most popular objects
    Popular(usize),
    /// Warm specific object list
    Explicit(Vec<String>),
    /// Warm all objects in bucket
    Bucket(String),
}

/// Cache warmer
pub struct CacheWarmer {
    cache: Arc<CacheLayer>,
    http_client: Client,
    origin_url: String,
}

impl CacheWarmer {
    /// Create new cache warmer
    pub fn new(cache: Arc<CacheLayer>, origin_url: String) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            cache,
            http_client,
            origin_url,
        }
    }

    /// Warm cache with strategy
    pub async fn warm(&self, strategy: WarmingStrategy) -> Result<usize> {
        match strategy {
            WarmingStrategy::Explicit(keys) => {
                self.warm_explicit(keys).await
            }
            WarmingStrategy::Popular(_count) => {
                info!("Popular warming not implemented yet");
                Ok(0)
            }
            WarmingStrategy::Bucket(_bucket) => {
                info!("Bucket warming not implemented yet");
                Ok(0)
            }
        }
    }

    /// Warm explicit list of objects
    async fn warm_explicit(&self, keys: Vec<String>) -> Result<usize> {
        info!("🔥 Warming cache with {} objects...", keys.len());
        
        let mut warmed = 0;
        
        for key in keys {
            match self.fetch_and_cache(&key).await {
                Ok(_) => {
                    warmed += 1;
                    info!("  ✅ Warmed: {}", key);
                }
                Err(e) => {
                    warn!("  ❌ Failed to warm {}: {}", key, e);
                }
            }
            
            // Small delay to avoid overwhelming origin
            sleep(Duration::from_millis(100)).await;
        }

        info!("🔥 Cache warming complete: {}/{} objects", warmed, warmed);
        
        Ok(warmed)
    }

    /// Fetch object from origin and cache it
    async fn fetch_and_cache(&self, key: &str) -> Result<()> {
        let url = format!("{}/{}", self.origin_url, key);
        
        let response = self.http_client.get(&url).send().await?;
        
        if response.status().is_success() {
            let bytes = response.bytes().await?;
            self.cache.put(key, bytes, None).await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Origin returned {}", response.status()))
        }
    }

    /// Start continuous warming (background task)
    pub async fn start_continuous(
        self: Arc<Self>,
        interval: Duration,
        strategy: WarmingStrategy,
    ) {
        tokio::spawn(async move {
            loop {
                sleep(interval).await;
                
                if let Err(e) = self.warm(strategy.clone()).await {
                    warn!("Cache warming failed: {}", e);
                }
            }
        });
    }
}

// Make WarmingStrategy cloneable
impl Clone for WarmingStrategy {
    fn clone(&self) -> Self {
        match self {
            WarmingStrategy::Popular(n) => WarmingStrategy::Popular(*n),
            WarmingStrategy::Explicit(v) => WarmingStrategy::Explicit(v.clone()),
            WarmingStrategy::Bucket(b) => WarmingStrategy::Bucket(b.clone()),
        }
    }
}
