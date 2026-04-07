use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub storage: StorageConfig,
    pub metadata: MetadataConfig,
    pub network: NetworkConfig,
    pub auth: AuthConfig,
    pub erasure: ErasureConfig,
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: Option<CacheConfig>,
    #[serde(default)]
    pub cdn: Option<CdnConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Data directory path
    pub data_dir: PathBuf,
    
    /// Enable direct I/O (O_DIRECT)
    pub direct_io: bool,
    
    /// Buffer alignment (typically 4096 for NVMe)
    pub buffer_alignment: usize,
    
    /// Enable CPU affinity (core pinning)
    pub cpu_affinity: bool,
    
    /// Number of IO threads
    pub io_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    /// Metadata database path
    pub db_path: PathBuf,
    
    /// Cache size in bytes
    pub cache_size: usize,
    
    /// Enable LSM tree
    pub use_lsm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address
    pub bind_address: String,
    
    /// Max concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in seconds
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key
    pub jwt_secret: String,
    
    /// JWT expiration in seconds
    pub jwt_expiration: i64,
    
    /// Shared secret with .NET backend
    pub shared_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureConfig {
    /// Number of data chunks
    pub data_chunks: usize,
    
    /// Number of parity chunks
    pub parity_chunks: usize,
    
    /// Enable SIMD acceleration
    pub use_simd: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
    
    /// Max database connections in pool
    pub max_connections: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                data_dir: PathBuf::from("../data"),
                direct_io: true,
                buffer_alignment: 4096,
                cpu_affinity: true,
                io_threads: 4,
            },
            metadata: MetadataConfig {
                db_path: PathBuf::from("../metadata"),
                cache_size: 1024 * 1024 * 1024, // 1GB
                use_lsm: true,
            },
            network: NetworkConfig {
                bind_address: "0.0.0.0:8080".to_string(),
                max_connections: 10000,
                timeout: 300,
            },
            auth: AuthConfig {
                jwt_secret: "change-me-in-production".to_string(),
                jwt_expiration: 3600,
                shared_secret: "change-me-in-production".to_string(),
            },
            erasure: ErasureConfig {
                data_chunks: 12,
                parity_chunks: 4,
                use_simd: true,
            },
            database: DatabaseConfig {
                url: "postgresql://husain:tt55oo77@31.97.217.73/barq_x30".to_string(),
                max_connections: 20,
            },
            cache: None, // Disabled by default, configure in config.toml if needed
            cdn: None,
        }
    }
}

pub fn load_config(path: &str) -> Result<Config> {
    let config_str = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => {
            // Create default config if not exists
            let default_config = Config::default();
            let config_str = toml::to_string_pretty(&default_config)?;
            std::fs::write(path, &config_str)?;
            return Ok(default_config);
        }
    };

    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub redis_url: String,
    pub memory_size: usize,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    pub enabled: bool,
    pub node_id: String,
    pub region: String,
    pub bind_port: u16,
    pub origin_url: String,
}
