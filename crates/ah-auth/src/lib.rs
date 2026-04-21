// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Agent Harbor Authentication
//!
//! Provides password hashing (Argon2id), JWT token management, secure random
//! token generation, OAuth provider types, and rate limiting for the platform
//! authentication layer.

pub mod jwt;
pub mod oauth;
pub mod password;
pub mod rate_limit;
pub mod tokens;

pub use jwt::{
    AccessTokenClaims, JwtConfig, RefreshTokenClaims, TokenPair, create_access_token,
    create_refresh_token, create_token_pair, validate_access_token, validate_refresh_token,
};
pub use oauth::{OAuthConfig, OAuthProvider, OAuthUserInfo};
pub use password::{hash_password, verify_password};
pub use rate_limit::{RateLimitError, RateLimiter};
pub use tokens::{
    generate_invitation_token, generate_reset_token, generate_token, generate_verification_token,
};
