//! CDN Edge Node implementation

use crate::cache::CacheLayer;
use crate::cdn::EdgeConfig;
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
};
use tracing::{info, warn, error};

/// CDN Edge Node
pub struct EdgeNode {
    config: EdgeConfig,
    cache: Arc<CacheLayer>,
    http_client: Client,
}

#[derive(Clone)]
struct EdgeState {
    cache: Arc<CacheLayer>,
    http_client: Client,
    origin_url: String,
}

impl EdgeNode {
    /// Create new edge node
    pub async fn new(config: EdgeConfig) -> Result<Self> {
        info!("🌐 Starting CDN Edge Node: {}", config.node_id);
        info!("   Region: {}", config.region);
        info!("   Bind: {}", config.bind_addr);
        info!("   Origin: {}", config.origin_url);

        let cache = Arc::new(
            CacheLayer::new(
                &config.redis_url,
                config.memory_cache_size,
                Duration::from_secs(config.cache_ttl),
            )
            .await?,
        );

        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config,
            cache,
            http_client,
        })
    }

    /// Start edge node server
    pub async fn serve(self) -> Result<()> {
        let state = EdgeState {
            cache: self.cache.clone(),
            http_client: self.http_client.clone(),
            origin_url: self.config.origin_url.clone(),
        };

        let app = Router::new()
            .route("/:bucket/:key", get(serve_object))
            .route("/_health", get(health_check))
            .route("/_stats", get(cache_stats))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(&self.config.bind_addr).await?;
        
        info!("🚀 CDN Edge Node listening on {}", self.config.bind_addr);

        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Serve object from cache or origin
async fn serve_object(
    State(state): State<EdgeState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Response {
    let cache_key = format!("{}/{}", bucket, key);

    // Try cache first
    if let Some(data) = state.cache.get(&cache_key).await {
        info!("📦 Serving from cache: {}", cache_key);
        return (
            StatusCode::OK,
            [(axum::http::header::CACHE_CONTROL, "public, max-age=3600")],
            data,
        ).into_response();
    }

    // Fetch from origin
    let origin_url = format!("{}/{}/{}", state.origin_url, bucket, key);
    
    match state.http_client.get(&origin_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        info!("🌐 Fetched from origin: {}", cache_key);
                        
                        // Cache for next request
                        let bytes_clone = bytes.clone();
                        let cache = state.cache.clone();
                        let cache_key_clone = cache_key.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = cache.put(&cache_key_clone, bytes_clone, None).await {
                                error!("Failed to cache object: {}", e);
                            }
                        });

                        (
                            StatusCode::OK,
                            [(axum::http::header::CACHE_CONTROL, "public, max-age=3600")],
                            bytes,
                        ).into_response()
                    }
                    Err(e) => {
                        error!("Failed to read origin response: {}", e);
                        (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read origin").into_response()
                    }
                }
            } else {
                warn!("Origin returned {}: {}", response.status(), cache_key);
                (response.status(), "Not found").into_response()
            }
        }
        Err(e) => {
            error!("Failed to fetch from origin: {}", e);
            (StatusCode::BAD_GATEWAY, "Origin unavailable").into_response()
        }
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Cache statistics endpoint
async fn cache_stats(State(state): State<EdgeState>) -> impl IntoResponse {
    let stats = state.cache.stats().await;
    (StatusCode::OK, axum::Json(stats))
}
