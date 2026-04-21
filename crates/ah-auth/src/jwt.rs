// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! JWT access and refresh token management.

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Token creation failed: {0}")]
    Creation(String),

    #[error("Token validation failed: {0}")]
    Validation(String),

    #[error("Token expired")]
    Expired,
}

/// JWT configuration.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// HMAC secret used to sign/verify tokens.
    pub secret: String,
    /// Access token time-to-live in seconds (default: 900 = 15 min).
    pub access_token_ttl_secs: u64,
    /// Refresh token time-to-live in seconds (default: 604800 = 7 days).
    pub refresh_token_ttl_secs: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: String::new(),
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 604_800,
        }
    }
}

/// Claims embedded in an access token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessTokenClaims {
    /// Subject — the user ID.
    pub sub: String,
    /// User email.
    pub email: String,
    /// Organisation / tenant ID (optional).
    pub org_id: Option<String>,
    /// Role within the organisation (optional).
    pub role: Option<String>,
    /// Expiration time (UNIX timestamp).
    pub exp: i64,
    /// Issued-at time (UNIX timestamp).
    pub iat: i64,
}

/// Claims embedded in a refresh token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RefreshTokenClaims {
    /// Subject — the user ID.
    pub sub: String,
    /// Unique identifier for this refresh token (allows rotation / revocation).
    pub token_id: String,
    /// Expiration time (UNIX timestamp).
    pub exp: i64,
    /// Issued-at time (UNIX timestamp).
    pub iat: i64,
}

/// An access + refresh token pair returned on sign-in / refresh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

/// Create a signed access token.
pub fn create_access_token(
    config: &JwtConfig,
    claims: &AccessTokenClaims,
) -> Result<String, JwtError> {
    let key = EncodingKey::from_secret(config.secret.as_bytes());
    encode(&Header::default(), claims, &key).map_err(|e| JwtError::Creation(e.to_string()))
}

/// Create a signed refresh token.
pub fn create_refresh_token(
    config: &JwtConfig,
    claims: &RefreshTokenClaims,
) -> Result<String, JwtError> {
    let key = EncodingKey::from_secret(config.secret.as_bytes());
    encode(&Header::default(), claims, &key).map_err(|e| JwtError::Creation(e.to_string()))
}

/// Validate an access token and return its claims.
pub fn validate_access_token(
    config: &JwtConfig,
    token: &str,
) -> Result<AccessTokenClaims, JwtError> {
    let key = DecodingKey::from_secret(config.secret.as_bytes());
    let validation = Validation::default();
    let data =
        decode::<AccessTokenClaims>(token, &key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
            _ => JwtError::Validation(e.to_string()),
        })?;
    Ok(data.claims)
}

/// Validate a refresh token and return its claims.
pub fn validate_refresh_token(
    config: &JwtConfig,
    token: &str,
) -> Result<RefreshTokenClaims, JwtError> {
    let key = DecodingKey::from_secret(config.secret.as_bytes());
    let validation = Validation::default();
    let data =
        decode::<RefreshTokenClaims>(token, &key, &validation).map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::Expired,
            _ => JwtError::Validation(e.to_string()),
        })?;
    Ok(data.claims)
}

/// Convenience: create an access + refresh token pair for a user.
pub fn create_token_pair(
    config: &JwtConfig,
    user_id: &str,
    email: &str,
    org_id: Option<String>,
    role: Option<String>,
) -> Result<TokenPair, JwtError> {
    let now = Utc::now().timestamp();

    let access_claims = AccessTokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        org_id,
        role,
        exp: now + config.access_token_ttl_secs as i64,
        iat: now,
    };

    let refresh_claims = RefreshTokenClaims {
        sub: user_id.to_string(),
        token_id: crate::tokens::generate_token(16),
        exp: now + config.refresh_token_ttl_secs as i64,
        iat: now,
    };

    let access_token = create_access_token(config, &access_claims)?;
    let refresh_token = create_refresh_token(config, &refresh_claims)?;

    Ok(TokenPair {
        access_token,
        refresh_token,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JwtConfig {
        JwtConfig {
            secret: "test-secret-that-is-long-enough-for-hmac".to_string(),
            access_token_ttl_secs: 900,
            refresh_token_ttl_secs: 604_800,
        }
    }

    #[test]
    fn test_jwt_token_lifecycle() {
        let config = test_config();
        let now = Utc::now().timestamp();

        let claims = AccessTokenClaims {
            sub: "user-123".to_string(),
            email: "alice@example.com".to_string(),
            org_id: Some("org-456".to_string()),
            role: Some("admin".to_string()),
            exp: now + 900,
            iat: now,
        };

        let token = create_access_token(&config, &claims).expect("create");
        let decoded = validate_access_token(&config, &token).expect("validate");

        assert_eq!(decoded.sub, "user-123");
        assert_eq!(decoded.email, "alice@example.com");
        assert_eq!(decoded.org_id, Some("org-456".to_string()));
        assert_eq!(decoded.role, Some("admin".to_string()));
    }

    #[test]
    fn test_jwt_refresh_token() {
        let config = test_config();
        let now = Utc::now().timestamp();

        let claims = RefreshTokenClaims {
            sub: "user-123".to_string(),
            token_id: "tid-abc".to_string(),
            exp: now + 604_800,
            iat: now,
        };

        let token = create_refresh_token(&config, &claims).expect("create");
        let decoded = validate_refresh_token(&config, &token).expect("validate");

        assert_eq!(decoded.sub, "user-123");
        assert_eq!(decoded.token_id, "tid-abc");
    }

    #[test]
    fn test_jwt_expired_token() {
        let config = test_config();

        let claims = AccessTokenClaims {
            sub: "user-123".to_string(),
            email: "alice@example.com".to_string(),
            org_id: None,
            role: None,
            exp: 1_000_000, // far in the past
            iat: 999_000,
        };

        let token = create_access_token(&config, &claims).expect("create");
        let err = validate_access_token(&config, &token).unwrap_err();
        assert!(matches!(err, JwtError::Expired));
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let config = test_config();
        let now = Utc::now().timestamp();

        let claims = AccessTokenClaims {
            sub: "user-123".to_string(),
            email: "alice@example.com".to_string(),
            org_id: None,
            role: None,
            exp: now + 900,
            iat: now,
        };

        let token = create_access_token(&config, &claims).expect("create");

        // Tamper with the token
        let tampered = format!("{}x", token);

        let err = validate_access_token(&config, &tampered).unwrap_err();
        assert!(matches!(err, JwtError::Validation(_)));
    }
}
