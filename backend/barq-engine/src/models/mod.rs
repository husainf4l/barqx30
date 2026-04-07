//! Database models for BARQ X30

use serde::{Deserialize, Serialize};

/// API request models
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub storage_quota: i64,
    pub storage_used: i64,
    pub created_at: chrono::NaiveDateTime,
}

impl From<crate::entities::users::Model> for UserResponse {
    fn from(user: crate::entities::users::Model) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            storage_quota: user.storage_quota,
            storage_used: user.storage_used,
            created_at: user.created_at,
        }
    }
}
