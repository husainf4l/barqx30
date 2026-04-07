-- Initial database schema
-- Create users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    storage_quota BIGINT NOT NULL DEFAULT 10737418240, -- 10GB default
    storage_used BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE INDEX idx_users_email ON users(email);

-- Create buckets table
CREATE TABLE buckets (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, name)
);

CREATE INDEX idx_buckets_user_id ON buckets(user_id);

-- Create objects table
CREATE TABLE objects (
    id BIGSERIAL PRIMARY KEY,
    bucket_id BIGINT NOT NULL REFERENCES buckets(id) ON DELETE CASCADE,
    key VARCHAR(1024) NOT NULL,
    size BIGINT NOT NULL,
    etag VARCHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL DEFAULT 'application/octet-stream',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(bucket_id, key)
);

CREATE INDEX idx_objects_bucket_id ON objects(bucket_id);
CREATE INDEX idx_objects_key ON objects(key);

-- Create sessions table
CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
