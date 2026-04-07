//! Database connection and operations using SeaORM

use crate::config::Config;
use crate::entities::{users, buckets, objects, sessions};
use anyhow::Result;
use sea_orm::{Database as SeaDatabase, DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use tracing::info;

/// Database manager
#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    #[allow(dead_code)]
    /// Connect to PostgreSQL database
    pub async fn connect(config: &Config) -> Result<Self> {
        info!("Connecting to PostgreSQL at: {}", config.database.url);
        
        let conn = SeaDatabase::connect(&config.database.url).await?;
        
        info!("✅ PostgreSQL connected");
        
        Ok(Self { conn })
    }

    /// Create database tables
    pub async fn migrate(&self) -> Result<()> {
        info!("Creating database tables...");
        
        use sea_orm::{Statement, ConnectionTrait};
        let db = &self.conn;
        
        // Create tables using raw SQL
        db.execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                name VARCHAR(255) NOT NULL,
                storage_quota BIGINT NOT NULL,
                storage_used BIGINT NOT NULL DEFAULT 0,
                role VARCHAR(50) NOT NULL DEFAULT 'user',
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            "#.to_string()
        )).await?;

        db.execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            r#"
            CREATE TABLE IF NOT EXISTS buckets (
                id SERIAL PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                name VARCHAR(255) NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            "#.to_string()
        )).await?;

        db.execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                id SERIAL PRIMARY KEY,
                bucket_id INTEGER NOT NULL REFERENCES buckets(id),
                key VARCHAR(512) NOT NULL UNIQUE,
                size BIGINT NOT NULL,
                etag VARCHAR(255) NOT NULL,
                content_type VARCHAR(255) NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            "#.to_string()
        )).await?;

        db.execute(Statement::from_string(
            sea_orm::DbBackend::Postgres,
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id VARCHAR(255) PRIMARY KEY,
                user_id INTEGER NOT NULL REFERENCES users(id),
                expires_at TIMESTAMP NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT NOW()
            );
            "#.to_string()
        )).await?;
        
        info!("✅ Tables created");
        Ok(())
    }

    /// Get database connection
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    // User operations
    pub async fn create_user(&self, email: &str, password_hash: &str, name: &str) -> Result<users::Model> {
        self.create_user_with_quota(email, password_hash, name, 10_737_418_240_i64).await
    }

    pub async fn create_user_with_quota(&self, email: &str, password_hash: &str, name: &str, storage_quota: i64) -> Result<users::Model> {
        let user = users::ActiveModel {
            email: Set(email.to_string()),
            password_hash: Set(password_hash.to_string()),
            name: Set(name.to_string()),
            storage_quota: Set(storage_quota),
            storage_used: Set(0),
            role: Set("user".to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let user = user.insert(&self.conn).await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<users::Model>> {
        let user = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.conn)
            .await?;
        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<users::Model>> {
        let user = users::Entity::find_by_id(id).one(&self.conn).await?;
        Ok(user)
    }

    pub async fn update_user_role(&self, id: i32, role: &str) -> Result<()> {
        let user = users::Entity::find_by_id(id).one(&self.conn).await?;
        if let Some(user) = user {
            let mut user: users::ActiveModel = user.into();
            user.role = Set(role.to_string());
            user.update(&self.conn).await?;
        }
        Ok(())
    }

    pub async fn update_storage_used(&self, id: i32, delta: i64) -> Result<()> {
        let user = users::Entity::find_by_id(id).one(&self.conn).await?;
        if let Some(user) = user {
            let mut user: users::ActiveModel = user.into();
            let new_used = user.storage_used.clone().unwrap() + delta;
            user.storage_used = Set(new_used);
            user.update(&self.conn).await?;
        }
        Ok(())
    }

    // Bucket operations
    pub async fn create_bucket(&self, user_id: i32, name: &str) -> Result<buckets::Model> {
        let bucket = buckets::ActiveModel {
            user_id: Set(user_id),
            name: Set(name.to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let bucket = bucket.insert(&self.conn).await?;
        Ok(bucket)
    }

    pub async fn get_bucket_by_name(&self, name: &str) -> Result<Option<buckets::Model>> {
        let bucket = buckets::Entity::find()
            .filter(buckets::Column::Name.eq(name))
            .one(&self.conn)
            .await?;
        Ok(bucket)
    }

    pub async fn list_user_buckets(&self, user_id: i32) -> Result<Vec<buckets::Model>> {
        let buckets = buckets::Entity::find()
            .filter(buckets::Column::UserId.eq(user_id))
            .all(&self.conn)
            .await?;
        Ok(buckets)
    }

    // Object operations
    pub async fn create_object(&self, bucket_id: i32, key: &str, size: i64, etag: &str, content_type: &str) -> Result<objects::Model> {
        // Find existing object
        if let Ok(Some(existing)) = self.get_object_by_key(key).await {
            let mut obj: objects::ActiveModel = existing.into();
            obj.size = Set(size);
            obj.etag = Set(etag.to_string());
            obj.content_type = Set(content_type.to_string());
            obj.created_at = Set(chrono::Utc::now().naive_utc());
            let updated = obj.update(&self.conn).await?;
            return Ok(updated);
        }

        let object = objects::ActiveModel {
            bucket_id: Set(bucket_id),
            key: Set(key.to_string()),
            size: Set(size),
            etag: Set(etag.to_string()),
            content_type: Set(content_type.to_string()),
            created_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };

        let object = object.insert(&self.conn).await?;
        Ok(object)
    }

    pub async fn get_object_by_key(&self, key: &str) -> Result<Option<objects::Model>> {
        let object = objects::Entity::find()
            .filter(objects::Column::Key.eq(key))
            .one(&self.conn)
            .await?;
        Ok(object)
    }

    pub async fn delete_object(&self, key: &str) -> Result<()> {
        objects::Entity::delete_many()
            .filter(objects::Column::Key.eq(key))
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn list_bucket_objects(&self, bucket_id: i32) -> Result<Vec<objects::Model>> {
        let objects = objects::Entity::find()
            .filter(objects::Column::BucketId.eq(bucket_id))
            .all(&self.conn)
            .await?;
        Ok(objects)
    }

    // Session operations
    pub async fn create_session(&self, id: &str, user_id: i32, expires_at: chrono::NaiveDateTime) -> Result<sessions::Model> {
        let session = sessions::ActiveModel {
            id: Set(id.to_string()),
            user_id: Set(user_id),
            expires_at: Set(expires_at),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        let session = session.insert(&self.conn).await?;
        Ok(session)
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<sessions::Model>> {
        let session = sessions::Entity::find_by_id(id.to_string()).one(&self.conn).await?;
        Ok(session)
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        sessions::Entity::delete_by_id(id.to_string()).exec(&self.conn).await?;
        Ok(())
    }
}
