//! S3-compatible API handlers

use super::AppState;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use tracing::{debug, error, info};

/// PUT /:bucket/:key - Upload object
pub async fn put_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    body: Bytes,
) -> Response {
    let full_key = format!("{}/{}", bucket, key);
    
    match state.storage.put_object(full_key.clone(), body.clone()).await {
        Ok(meta) => {
            debug!("PUT object: {} (ETag: {})", full_key, meta.etag);
            
            // Only cache small objects (e.g., < 10MB) to prevent memory & network exhaustion
            if body.len() < 10 * 1024 * 1024 {
                if let Some(ref cache) = state.cache {
                    let cache_key = format!("{}/{}", bucket, key);
                    if let Err(e) = cache.put(&cache_key, body.clone(), None).await {
                        error!("Failed to cache object {}: {}", full_key, e);
                    }
                }
            }
            
            // Save object metadata to database
            if let Ok(Some(bucket_model)) = state.db.get_bucket_by_name(&bucket).await {
                let size = body.len() as i64;
                let content_type = "application/octet-stream"; // Optional: could extract from headers
                
                // Try getting existing object to update size difference or just insert/update
                // create_object should handle upsert or we just insert for now
                if let Err(e) = state.db.create_object(bucket_model.id, &key, size, &meta.etag, content_type).await {
                    error!("Failed to save object metadata to database: {}", e);
                } else {
                    // Update user storage usage
                    if let Err(e) = state.db.update_storage_used(bucket_model.user_id, size).await {
                        error!("Failed to update user storage usage: {}", e);
                    }
                }
            } else {
                error!("Warning: Bucket {} not found in database. Metadata not saved.", bucket);
            }
            
            (
                StatusCode::OK,
                [("ETag", meta.etag.as_str())],
                "OK",
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to PUT object {}: {}", full_key, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /:bucket/:key - Download object (with caching)
pub async fn get_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Response {
    let full_key = format!("{}/{}", bucket, key);
    let cache_key = format!("{}/{}", bucket, key);
    
    // Try cache first if enabled
    if let Some(ref cache) = state.cache {
        if let Some(cached_data) = cache.get(&cache_key).await {
            info!("Cache HIT: {}", full_key);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                cached_data.to_vec()
            ).into_response();
        }
        info!("Cache MISS: {}", full_key);
    }
    
    // Cache miss or disabled - fetch from storage
    match state.storage.get_object(&full_key).await {
        Ok(data) => {
            debug!("GET object: {} ({} bytes)", full_key, data.len());
            
            // Cache the fetched object if cache is enabled
            if let Some(ref cache) = state.cache {
                let data_bytes = Bytes::from(data.clone());
                if let Err(e) = cache.put(&cache_key, data_bytes, None).await {
                    error!("Failed to cache object {}: {}", full_key, e);
                }
            }
            
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/octet-stream")],
                data
            ).into_response()
        }
        Err(e) => {
            error!("Failed to GET object {}: {}", full_key, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// DELETE /:bucket/:key - Delete object (with cache invalidation)
pub async fn delete_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Response {
    let full_key = format!("{}/{}", bucket, key);
    
    match state.storage.delete_object(&full_key).await {
        Ok(_) => {
            debug!("DELETE object: {}", full_key);
            
            // Update db
            if let Ok(Some(obj)) = state.db.get_object_by_key(&key).await {
                if let Err(e) = state.db.delete_object(&key).await {
                    error!("Failed to delete object from DB for {}: {}", key, e);
                } else {
                    if let Err(e) = state.db.update_storage_used(obj.bucket_id, -obj.size).await {
                        error!("Failed to decrement storage for {}: {}", bucket, e);
                    }
                }
            }

            // Invalidate cache if enabled
            if let Some(ref cache) = state.cache {
                let cache_key = format!("{}/{}", bucket, key);
                if let Err(e) = cache.invalidate(&cache_key).await {
                    error!("Failed to invalidate cache for {}: {}", full_key, e);
                }
            }
            
            (StatusCode::NO_CONTENT, "").into_response()
        }
        Err(e) => {
            error!("Failed to DELETE object {}: {}", full_key, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

/// GET /:bucket - List objects in bucket
pub async fn list_objects(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> Response {
    let objects = state.storage.list_objects().await;
    let bucket_objects: Vec<String> = objects
        .into_iter()
        .filter(|k| k.starts_with(&format!("{}/", bucket)))
        .collect();

    debug!("LIST bucket: {} ({} objects)", bucket, bucket_objects.len());

    // Simple JSON response (full S3 XML compatibility would go here)
    let json = serde_json::json!({
        "bucket": bucket,
        "objects": bucket_objects,
        "count": bucket_objects.len()
    });

    (StatusCode::OK, json.to_string()).into_response()
}

/// PUT /:bucket - Create bucket
pub async fn create_bucket(Path(bucket): Path<String>) -> Response {
    debug!("CREATE bucket: {}", bucket);
    (StatusCode::OK, "Bucket created").into_response()
}

/// DELETE /:bucket - Delete bucket
pub async fn delete_bucket(Path(bucket): Path<String>) -> Response {
    debug!("DELETE bucket: {}", bucket);
    (StatusCode::NO_CONTENT, "").into_response()
}
