//! CDN edge server module

#![allow(dead_code)]

pub mod edge_node;
pub mod geo_routing;
pub mod cache_warming;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// CDN edge node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeConfig {
    /// Node identifier
    pub node_id: String,
    /// Geographic region
    pub region: String,
    /// Listening address
    pub bind_addr: SocketAddr,
    /// Origin server URL
    pub origin_url: String,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
    /// Memory cache size (number of objects)
    pub memory_cache_size: usize,
    /// Redis URL for L2 cache
    pub redis_url: String,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            node_id: "edge-01".to_string(),
            region: "us-east-1".to_string(),
            bind_addr: "0.0.0.0:8081".parse().unwrap(),
            origin_url: "http://localhost:8080".to_string(),
            cache_ttl: 3600, // 1 hour
            memory_cache_size: 10000,
            redis_url: "redis://localhost:6379".to_string(),
        }
    }
}
