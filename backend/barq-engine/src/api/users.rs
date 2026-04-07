//! User management API endpoints

use crate::auth::Auth;
use crate::database::Database;
use crate::models::{LoginRequest, RegisterRequest, AuthResponse};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Database>,
    pub auth: Arc<Auth>,
}

/// POST /api/auth/register - Register new user
pub async fn register(
    State(state): State<ApiState>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    // Validate email format
    if !req.email.contains('@') {
        return (StatusCode::BAD_REQUEST, "Invalid email format").into_response();
    }

    // Check if user exists
    match state.db.get_user_by_email(&req.email).await {
        Ok(Some(_)) => {
            return (StatusCode::CONFLICT, "Email already registered").into_response();
        }
        Ok(None) => {}
        Err(e) => {
            error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    }

    // Hash password with Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(req.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(e) => {
            error!("Password hashing error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Password hashing failed").into_response();
        }
    };

    // Create user
    let user = match state.db.create_user(&req.email, &password_hash, &req.name).await {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to create user: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user").into_response();
        }
    };

    // Generate JWT token
    let token = match state.auth.generate_token(&user.id.to_string(), vec!["read".to_string(), "write".to_string()]) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response();
        }
    };

    // Create session
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
    if let Err(e) = state.db.create_session(&token, user.id, expires_at.naive_utc()).await {
        error!("Failed to create session: {}", e);
    }

    info!("User registered: {}", user.email);

    let response = AuthResponse {
        token,
        user: user.into(),
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

/// POST /api/auth/login - Login user
pub async fn login(
    State(state): State<ApiState>,
    Json(req): Json<LoginRequest>,
) -> Response {
    // Get user by email
    let user = match state.db.get_user_by_email(&req.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
        }
        Err(e) => {
            error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    // Verify password
    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to parse password hash: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Authentication error").into_response();
        }
    };

    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    // Generate JWT token
    let token = match state.auth.generate_token(&user.id.to_string(), vec!["read".to_string(), "write".to_string()]) {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to generate token: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token").into_response();
        }
    };

    // Create session
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
    if let Err(e) = state.db.create_session(&token, user.id, expires_at.naive_utc()).await {
        error!("Failed to create session: {}", e);
    }

    info!("User logged in: {}", user.email);

    let response = AuthResponse {
        token,
        user: user.into(),
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/users/me - Get current user info
pub async fn get_me(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> Response {
    // Extract token from Authorization header
    let token = match headers.get("authorization") {
        Some(value) => {
            let auth_str = match value.to_str() {
                Ok(s) => s,
                Err(_) => {
                    return (StatusCode::BAD_REQUEST, "Invalid authorization header").into_response();
                }
            };
            
            // Remove "Bearer " prefix if present
            if auth_str.starts_with("Bearer ") {
                &auth_str[7..]
            } else {
                auth_str
            }
        }
        None => {
            return (StatusCode::UNAUTHORIZED, "No authorization token provided").into_response();
        }
    };

    // Validate token and extract user_id
    let claims = match state.auth.validate_token(token) {
        Ok(claims) => claims,
        Err(e) => {
            error!("Token validation failed: {}", e);
            return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
        }
    };

    // Parse user_id from claims.sub
    let user_id: i32 = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid user_id in token: {}", claims.sub);
            return (StatusCode::UNAUTHORIZED, "Invalid user_id in token").into_response();
        }
    };

    // Get user from database
    let user = match state.db.get_user_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        }
        Err(e) => {
            error!("Database error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response();
        }
    };

    info!("User info retrieved: {}", user.email);

    (StatusCode::OK, Json(user)).into_response()
}
