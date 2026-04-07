//! Database connection and operations

use crate::config::Config;
use crate::models::{User, Bucket, Object, Session};
use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

/// Database manager
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Connect to PostgreSQL database
    pub async fn connect(config: &Config) -> Result<Self> {
        info!("Connecting to PostgreSQL at: {}", config.database.url);
        
        let pool = PgPool::connect(&config.database.url).await?;
        
        info!("✅ PostgreSQL connected");
        
        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        sqlx::migrate!("../migrations").run(&self.pool).await?;
        info!("✅ Migrations complete");
        Ok(())
    }

    /// Get connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // User operations
    pub async fn create_user(&self, email: &str, password_hash: &str, name: &str) -> Result<User> {
        self.create_user_with_quota(email, password_hash, name, 10_737_418_240_i64).await
    }

    pub async fn create_user_with_quota(&self, email: &str, password_hash: &str, name: &str, storage_quota: i64) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, name, storage_quota, storage_used, role)
            VALUES ($1, $2, $3, $4, 0, 'user')
            RETURNING *
            "#
        )
        .bind(email)
        .bind(password_hash)
        .bind(name)
        .bind(storage_quota)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn update_storage_used(&self, user_id: i64, bytes: i64) -> Result<()> {
        sqlx::query("UPDATE users SET storage_used = storage_used + $1 WHERE id = $2")
            .bind(bytes)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Session operations
    pub async fn create_session(&self, user_id: i64, token: &str, expires_at: chrono::DateTime<chrono::Utc>) -> Result<Session> {
        let session = sqlx::query_as::<_, Session>(
            "INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_session(&self, token: &str) -> Result<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE token = $1 AND expires_at > NOW()"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    // Bucket operations
    pub async fn create_bucket(&self, user_id: i64, name: &str) -> Result<Bucket> {
        let bucket = sqlx::query_as::<_, Bucket>(
            "INSERT INTO buckets (user_id, name) VALUES ($1, $2) RETURNING *"
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(bucket)
    }

    pub async fn get_user_buckets(&self, user_id: i64) -> Result<Vec<Bucket>> {
        let buckets = sqlx::query_as::<_, Bucket>(
            "SELECT * FROM buckets WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(buckets)
    }

    pub async fn get_bucket(&self, user_id: i64, name: &str) -> Result<Option<Bucket>> {
        let bucket = sqlx::query_as::<_, Bucket>(
            "SELECT * FROM buckets WHERE user_id = $1 AND name = $2"
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(bucket)
    }

    // Object operations
    pub async fn create_object(&self, bucket_id: i64, key: &str, size: i64, etag: &str, content_type: &str) -> Result<Object> {
        let object = sqlx::query_as::<_, Object>(
            r#"
            INSERT INTO objects (bucket_id, key, size, etag, content_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(bucket_id)
        .bind(key)
        .bind(size)
        .bind(etag)
        .bind(content_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(object)
    }

    pub async fn get_bucket_objects(&self, bucket_id: i64) -> Result<Vec<Object>> {
        let objects = sqlx::query_as::<_, Object>(
            "SELECT * FROM objects WHERE bucket_id = $1 ORDER BY created_at DESC"
        )
        .bind(bucket_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(objects)
    }

    pub async fn delete_object(&self, bucket_id: i64, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM objects WHERE bucket_id = $1 AND key = $2")
            .bind(bucket_id)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
