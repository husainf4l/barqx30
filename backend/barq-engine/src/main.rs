mod auth;
mod cache;
mod cdn;
mod cli;
mod config;
mod entities;
mod erasure;
mod metadata;
mod network;
mod storage;
mod database;
mod models;
mod api;

use anyhow::Result;
use clap::Parser;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "BARQ X30")]
#[command(author = "BARQ Team")]
#[command(version = "0.1.0")]
#[command(about = "Ultra-High Performance Object Storage Engine", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Server mode
    #[arg(short, long, default_value = "server")]
    mode: String,

    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    bind: String,
}

// Linux: Use standard tokio multi-thread runtime
#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() -> Result<()> {
    async_main().await
}

// macOS: Use standard tokio runtime
#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() -> Result<()> {
    async_main().await
}

// Other platforms: Use standard tokio runtime
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[tokio::main]
async fn main() -> Result<()> {
    async_main().await
}

async fn async_main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    info!("🚀 BARQ X30 Storage Engine Starting...");
    info!("⚡ Target Latency: 30 microseconds");
    info!("📁 Configuration: {}", args.config);
    info!("🌐 Binding to: {}", args.bind);
    
    // Display platform mode
    #[cfg(target_os = "linux")]
    info!("🏎️  Platform: Linux (io_uring ENABLED - Production Mode)");
    
    #[cfg(target_os = "macos")]
    warn!("🍎 Platform: macOS (tokio::fs - Development Mode)");

    // Load configuration
    let config = config::load_config(&args.config)?;
    info!("✅ Configuration loaded");

    // Initialize database
    info!("💾 Connecting to PostgreSQL...");
    let db = database::Database::connect(&config).await?;
    db.migrate().await?;
    info!("✅ Database connected");

    // Initialize storage engine
    info!("💾 Initializing storage engine...");
    let storage_engine = storage::StorageEngine::new(&config).await?;
    info!("✅ Storage engine initialized");

    // Initialize metadata store
    info!("🗂️  Initializing metadata store...");
    let metadata_store = metadata::MetadataStore::new(&config)?;
    info!("✅ Metadata store initialized");

    // Initialize authentication
    info!("🔐 Initializing authentication...");
    let auth = auth::Auth::new(&config)?;
    info!("✅ Authentication initialized");

    // Initialize cache manager
    let cache_manager = if let Some(ref cache_cfg) = config.cache {
        if cache_cfg.enabled {
            info!("🗄️  Initializing cache manager...");
            let cm = cache::CacheLayer::new(
                &cache_cfg.redis_url,
                cache_cfg.memory_size,
                Duration::from_secs(cache_cfg.ttl_seconds),
            ).await?;
            info!("✅ Cache manager initialized");
            Some(Arc::new(cm))
        } else {
            info!("ℹ️  Cache disabled");
            None
        }
    } else {
        info!("ℹ️  Cache not configured");
        None
    };

    // Start network server — prefer config file bind address, fall back to CLI arg
    let bind_addr = if args.bind != "0.0.0.0:8080" {
        args.bind.clone()
    } else {
        config.network.bind_address.clone()
    };
    info!("🌐 Starting network server...");
    network::start_server(&bind_addr, storage_engine, metadata_store, auth, db, cache_manager).await?;

    Ok(())
}

