// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! OAuth provider configuration types.
//!
//! Struct-only definitions for now; actual OAuth flows will be added in M2+.

use serde::{Deserialize, Serialize};

/// Supported OAuth identity providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OAuthProvider {
    GitHub,
    Google,
    GitLab,
}

/// Configuration for an OAuth provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// User information returned by an OAuth provider after authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    /// Provider-specific user identifier.
    pub provider_id: String,
    /// User email address.
    pub email: String,
    /// Display name.
    pub name: Option<String>,
    /// Avatar / profile image URL.
    pub avatar_url: Option<String>,
}
