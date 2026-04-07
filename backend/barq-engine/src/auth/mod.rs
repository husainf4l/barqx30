//! JWT-based authentication and authorization
//!
//! Validates tokens issued by the .NET management layer
//! using shared secret cryptography.

use crate::config::Config;
use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// JWT claims structure
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub exp: i64,     // Expiration timestamp
    pub iat: i64,     // Issued at timestamp
    pub bucket: Option<String>,  // Optional bucket scope
    pub permissions: Vec<String>, // Permissions: read, write, delete
}

/// Authentication manager
#[allow(dead_code)]
#[derive(Clone)]
pub struct Auth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

#[allow(dead_code)]
impl Auth {
    /// Creates a new authentication manager
    pub fn new(config: &Config) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.auth.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.auth.jwt_secret.as_bytes());
        
        let validation = Validation::default();

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Validates a JWT token
    ///
    /// # Arguments
    /// * `token` - JWT token string
    ///
    /// # Returns
    /// Decoded claims if valid
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| anyhow!("Token validation failed: {}", e))?;

        debug!("Token validated for user: {}", token_data.claims.sub);
        Ok(token_data.claims)
    }

    /// Generates a new JWT token (for testing/development)
    pub fn generate_token(&self, user_id: &str, permissions: Vec<String>) -> Result<String> {
        let now = chrono::Utc::now().timestamp();
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 3600, // 1 hour expiration
            iat: now,
            bucket: None,
            permissions,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Token generation failed: {}", e))?;

        Ok(token)
    }

    /// Checks if the claims have the required permission
    pub fn has_permission(claims: &Claims, required: &str) -> bool {
        claims.permissions.iter().any(|p| p == required || p == "*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AuthConfig;

    #[test]
    fn test_token_generation_and_validation() {
        let config = Config {
            auth: AuthConfig {
                jwt_secret: "test_secret_key_12345".to_string(),
                jwt_expiration: 3600,
                shared_secret: "shared_secret".to_string(),
            },
            ..Default::default()
        };

        let auth = Auth::new(&config).unwrap();

        // Generate token
        let token = auth
            .generate_token("user123", vec!["read".to_string(), "write".to_string()])
            .unwrap();

        // Validate token
        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert!(Auth::has_permission(&claims, "read"));
        assert!(Auth::has_permission(&claims, "write"));
        assert!(!Auth::has_permission(&claims, "delete"));
    }
}
