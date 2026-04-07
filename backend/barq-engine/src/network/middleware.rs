//! HTTP middleware for authentication and metrics

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};

/// JWT authentication middleware
#[allow(dead_code)]
pub async fn auth_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    // Skip auth for health check
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // Validate token
    if let Some(auth_str) = auth_header {
        if auth_str.starts_with("Bearer ") {
            let _token = &auth_str[7..];
            // Token validation would go here
            // For now, accept any Bearer token
            return Ok(next.run(request).await);
        }
    }

    // No valid token
    Err(StatusCode::UNAUTHORIZED)
}
