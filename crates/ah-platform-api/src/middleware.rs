// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Auth middleware helpers for extracting the authenticated user from requests
//! and checking org-level permissions.

use crate::auth_handlers::AuthError;
use ah_auth::JwtConfig;
use axum::http::HeaderMap;
use sqlx::Row;

/// Extract user_id from JWT Authorization header.
pub fn extract_auth_user(headers: &HeaderMap, jwt_config: &JwtConfig) -> Result<String, AuthError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            AuthError::Unauthorized("Missing or invalid Authorization header".to_string())
        })?;

    let claims = ah_auth::validate_access_token(jwt_config, token)
        .map_err(|_| AuthError::Unauthorized("Invalid or expired token".to_string()))?;

    Ok(claims.sub)
}

/// Check if user is a member of the org. Returns the member's role.
pub async fn get_member_role(
    pool: &sqlx::AnyPool,
    user_id: &str,
    org_id: &str,
) -> Result<String, AuthError> {
    let row = sqlx::query("SELECT role FROM org_members WHERE org_id = $1 AND user_id = $2")
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let role: String = r.get("role");
            Ok(role)
        }
        None => Err(AuthError::Forbidden(
            "Not a member of this organization".to_string(),
        )),
    }
}

/// Check that user is an admin of the org. Returns error if not.
pub async fn require_admin(
    pool: &sqlx::AnyPool,
    user_id: &str,
    org_id: &str,
) -> Result<(), AuthError> {
    let role = get_member_role(pool, user_id, org_id).await?;
    if role != "admin" {
        return Err(AuthError::Forbidden(
            "Only admins can perform this action".to_string(),
        ));
    }
    Ok(())
}
