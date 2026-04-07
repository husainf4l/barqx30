//! Bucket management API endpoints

use crate::api::ApiState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct BucketResponse {
    pub id: i32,
    pub name: String,
    pub user_id: i32,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct BucketListResponse {
    pub buckets: Vec<BucketResponse>,
    pub count: usize,
}

/// POST /api/buckets - Create a new bucket
pub async fn create_bucket(
    State(state): State<ApiState>,
    Json(req): Json<CreateBucketRequest>,
) -> Response {
    // TODO: Get user_id from JWT token
    let user_id = 1; // Hardcoded for now
    
    match state.db.create_bucket(user_id, &req.name).await {
        Ok(bucket) => {
            debug!("Bucket created: {}", bucket.name);
            let response = BucketResponse {
                id: bucket.id,
                name: bucket.name,
                user_id: bucket.user_id,
                created_at: bucket.created_at.to_string(),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to create bucket: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /api/buckets - List all buckets for current user
pub async fn list_buckets(State(state): State<ApiState>) -> Response {
    // TODO: Get user_id from JWT token
    let user_id = 1; // Hardcoded for now
    
    match state.db.list_user_buckets(user_id).await {
        Ok(buckets) => {
            let response = BucketListResponse {
                count: buckets.len(),
                buckets: buckets.into_iter().map(|b| BucketResponse {
                    id: b.id,
                    name: b.name,
                    user_id: b.user_id,
                    created_at: b.created_at.to_string(),
                }).collect(),
            };
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to list buckets: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /api/buckets/:name - Get bucket details
pub async fn get_bucket(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Response {
    match state.db.get_bucket_by_name(&name).await {
        Ok(Some(bucket)) => {
            let response = BucketResponse {
                id: bucket.id,
                name: bucket.name,
                user_id: bucket.user_id,
                created_at: bucket.created_at.to_string(),
            };
            Json(response).into_response()
        }
        Ok(None) => {
            (StatusCode::NOT_FOUND, "Bucket not found").into_response()
        }
        Err(e) => {
            error!("Failed to get bucket: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// GET /api/buckets/:name/objects - List objects in bucket
pub async fn list_bucket_objects(
    State(state): State<ApiState>,
    Path(name): Path<String>,
) -> Response {
    // Get bucket first
    let bucket = match state.db.get_bucket_by_name(&name).await {
        Ok(Some(b)) => b,
        Ok(None) => return (StatusCode::NOT_FOUND, "Bucket not found").into_response(),
        Err(e) => {
            error!("Failed to get bucket: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    match state.db.list_bucket_objects(bucket.id).await {
        Ok(objects) => {
            let count = objects.len();
            let obj_list: Vec<_> = objects.into_iter().map(|o| serde_json::json!({
                "key": o.key,
                "size": o.size,
                "etag": o.etag,
                "content_type": o.content_type,
                "created_at": o.created_at.to_string(),
            })).collect();
            
            let response = serde_json::json!({
                "bucket": name,
                "objects": obj_list,
                "count": count,
            });
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to list objects: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
