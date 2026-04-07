//! Network server with S3-compatible API
//!
//! Provides HTTP/HTTPS endpoints compatible with AWS S3,
//! allowing drop-in replacement for existing applications.

mod s3_handlers;
mod middleware;

use crate::auth::Auth;
use crate::cache::CacheLayer;
use crate::metadata::MetadataStore;
use crate::storage::StorageEngine;
use crate::database::Database;
use crate::api;
use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

/// Shared application state
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<StorageEngine>,
    pub metadata: Arc<MetadataStore>,
    pub auth: Arc<Auth>,
    pub cache: Option<Arc<CacheLayer>>,
    pub db: Arc<Database>,
}

/// Starts the HTTP server
pub async fn start_server(
    bind_addr: &str,
    storage: StorageEngine,
    metadata: MetadataStore,
    auth: Auth,
    db: Database,
    cache: Option<Arc<CacheLayer>>,
) -> Result<()> {
    let db = Arc::new(db);
    
    let state = AppState {
        storage: Arc::new(storage),
        metadata: Arc::new(metadata),
        auth: Arc::new(auth.clone()),
        cache,
        db: db.clone(),
    };

    let api_state = api::ApiState {
        db: db.clone(),
        auth: Arc::new(auth),
    };

    // Build router with S3-compatible endpoints + API endpoints
    let api_routes = Router::new()
        // Auth endpoints
        .route("/api/auth/register", post(api::users::register))
        .route("/api/auth/login", post(api::users::login))
        .route("/api/users/me", get(api::users::get_me))
        // Bucket management endpoints (REST API)
        .route("/api/buckets", post(api::buckets::create_bucket))
        .route("/api/buckets", get(api::buckets::list_buckets))
        .route("/api/buckets/{name}", get(api::buckets::get_bucket))
        .route("/api/buckets/{name}/objects", get(api::buckets::list_bucket_objects))
        .with_state(api_state);

    let s3_routes = Router::new()
        .route("/{bucket}", get(s3_handlers::list_objects))
        .route("/{bucket}", put(s3_handlers::create_bucket))
        .route("/{bucket}", delete(s3_handlers::delete_bucket))
        .route("/{bucket}/{key}", put(s3_handlers::put_object))
        .route("/{bucket}/{key}", get(s3_handlers::get_object))
        .route("/{bucket}/{key}", delete(s3_handlers::delete_object))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)) // 10GB max upload size
        .with_state(state.clone());

    let app = Router::new()
        .merge(api_routes)
        .merge(s3_routes)
        .route("/health", get(health_check))
        .route("/cache/stats", get(cache_stats))
        .route("/cache/purge", post(cache_purge))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http());

    info!("🚀 BARQ X30 server listening on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "BARQ X30 - OK"
}

async fn cache_stats(axum::extract::State(state): axum::extract::State<AppState>) -> axum::response::Response {
    if let Some(ref cache) = state.cache {
        let stats = cache.stats().await;
        axum::Json(stats).into_response()
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Cache not enabled").into_response()
    }
}

async fn cache_purge(axum::extract::State(state): axum::extract::State<AppState>) -> axum::response::Response {
    if let Some(ref cache) = state.cache {
        let _ = cache.clear().await;
        "Cache purged".into_response()
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Cache not enabled").into_response()
    }
}
