-- Add role column to users table
ALTER TABLE users ADD COLUMN role VARCHAR(50) NOT NULL DEFAULT 'user';

CREATE INDEX idx_users_role ON users(role);
