// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Authentication API handlers (sign-up, sign-in, token refresh, password
//! reset, email verification).

use crate::PlatformApiState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::{error, info};

// =============================================================================
// Request / response types
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupResponse {
    pub user_id: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordConfirmRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// =============================================================================
// Error type
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Auth error: {0}")]
    Internal(String),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AuthError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AuthError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg.clone()),
            AuthError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg.clone()),
            AuthError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg.clone()),
            AuthError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AuthError::Database(err) => {
                error!("Database error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "Internal server error".to_string(),
                )
            }
            AuthError::Internal(msg) => {
                error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(ErrorBody {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// POST /api/v1/auth/signup
pub async fn signup(
    State(state): State<PlatformApiState>,
    Json(req): Json<SignupRequest>,
) -> Result<Response, AuthError> {
    // Basic validation
    if req.email.is_empty() || !req.email.contains('@') {
        return Err(AuthError::BadRequest("Invalid email".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AuthError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check duplicate email
    let existing = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await?;

    if existing.is_some() {
        return Err(AuthError::Conflict(
            "A user with this email already exists".to_string(),
        ));
    }

    // Hash password
    let password_hash =
        ah_auth::hash_password(&req.password).map_err(|e| AuthError::Internal(e.to_string()))?;

    let user_id = uuid::Uuid::new_v4().to_string();
    let verification_token = ah_auth::generate_verification_token();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Insert user
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, email_verified, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 0, $5, $6)",
    )
    .bind(&user_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.name)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // Store verification token in the invitations table (type = 'email_verification').
    // We reuse the invitations table with a sentinel org_id.
    let token_id = uuid::Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "INSERT INTO invitations (id, org_id, email, role, token, invited_by, expires_at)
         VALUES ($1, $2, $3, 'email_verification', $4, NULL, $5)",
    )
    .bind(&token_id)
    .bind(&user_id) // store user_id as org_id for lookup convenience
    .bind(&req.email)
    .bind(&verification_token)
    .bind(&expires_at)
    .execute(&state.pool)
    .await?;

    // In a real deployment email would be sent here. For now, just log.
    info!(
        user_id = %user_id,
        email = %req.email,
        "Verification email would be sent (token={})",
        verification_token,
    );

    Ok((
        StatusCode::CREATED,
        Json(SignupResponse {
            user_id,
            message: "Verification email sent".to_string(),
        }),
    )
        .into_response())
}

/// POST /api/v1/auth/signin
pub async fn signin(
    State(state): State<PlatformApiState>,
    Json(req): Json<SigninRequest>,
) -> Result<Json<SigninResponse>, AuthError> {
    let row = sqlx::query(
        "SELECT id, email, password_hash, name, email_verified FROM users WHERE email = $1",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AuthError::Unauthorized("Invalid credentials".to_string()))?;

    let password_hash: Option<String> = row.try_get("password_hash").ok().flatten();
    let password_hash =
        password_hash.ok_or_else(|| AuthError::Unauthorized("Invalid credentials".to_string()))?;

    let valid = ah_auth::verify_password(&req.password, &password_hash)
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    if !valid {
        return Err(AuthError::Unauthorized("Invalid credentials".to_string()));
    }

    let user_id: String = row.get("id");
    let email: String = row.get("email");
    let name: Option<String> = row.try_get("name").ok().flatten();

    let pair = ah_auth::create_token_pair(&state.jwt_config, &user_id, &email, None, None)
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    info!(user_id = %user_id, "User signed in");

    Ok(Json(SigninResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
        user: UserInfo {
            id: user_id,
            email,
            name,
        },
    }))
}

/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<PlatformApiState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AuthError> {
    let claims = ah_auth::validate_refresh_token(&state.jwt_config, &req.refresh_token)
        .map_err(|e| AuthError::Unauthorized(format!("Invalid refresh token: {}", e)))?;

    // Look up user to build new access token claims
    let row = sqlx::query("SELECT id, email FROM users WHERE id = $1")
        .bind(&claims.sub)
        .fetch_optional(&state.pool)
        .await?;

    let row = row.ok_or_else(|| AuthError::Unauthorized("User not found".to_string()))?;

    let user_id: String = row.get("id");
    let email: String = row.get("email");

    let pair = ah_auth::create_token_pair(&state.jwt_config, &user_id, &email, None, None)
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    info!(user_id = %user_id, "Token refreshed");

    Ok(Json(RefreshResponse {
        access_token: pair.access_token,
        refresh_token: pair.refresh_token,
    }))
}

/// POST /api/v1/auth/reset-password
pub async fn reset_password(
    State(state): State<PlatformApiState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    // Always return success to avoid email enumeration
    let row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&req.email)
        .fetch_optional(&state.pool)
        .await?;

    if let Some(row) = row {
        let user_id: String = row.get("id");
        let reset_token = ah_auth::generate_reset_token();
        let token_id = uuid::Uuid::new_v4().to_string();
        let expires_at = (chrono::Utc::now() + chrono::Duration::hours(1))
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();

        sqlx::query(
            "INSERT INTO invitations (id, org_id, email, role, token, invited_by, expires_at)
             VALUES ($1, $2, $3, 'password_reset', $4, NULL, $5)",
        )
        .bind(&token_id)
        .bind(&user_id)
        .bind(&req.email)
        .bind(&reset_token)
        .bind(&expires_at)
        .execute(&state.pool)
        .await?;

        info!(
            user_id = %user_id,
            "Password reset email would be sent (token={})",
            reset_token,
        );
    }

    Ok(Json(MessageResponse {
        message: "If that email exists, a reset link has been sent".to_string(),
    }))
}

/// POST /api/v1/auth/reset-password/confirm
pub async fn reset_password_confirm(
    State(state): State<PlatformApiState>,
    Json(req): Json<ResetPasswordConfirmRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    if req.new_password.len() < 8 {
        return Err(AuthError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Find the reset token
    let row = sqlx::query(
        "SELECT id, org_id, expires_at, accepted_at
         FROM invitations
         WHERE token = $1 AND role = 'password_reset'",
    )
    .bind(&req.token)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AuthError::BadRequest("Invalid or expired token".to_string()))?;

    // Check if already used
    let accepted_at: Option<String> = row.try_get("accepted_at").ok().flatten();
    if accepted_at.is_some() {
        return Err(AuthError::BadRequest("Token already used".to_string()));
    }

    // Check expiry
    let expires_at: String = row.get("expires_at");
    if expires_at < now_str {
        return Err(AuthError::BadRequest("Token has expired".to_string()));
    }

    let user_id: String = row.get("org_id"); // we stored user_id in org_id
    let invitation_id: String = row.get("id");

    // Hash new password
    let password_hash = ah_auth::hash_password(&req.new_password)
        .map_err(|e| AuthError::Internal(e.to_string()))?;

    // Update user password
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
        .bind(&password_hash)
        .bind(&now_str)
        .bind(&user_id)
        .execute(&state.pool)
        .await?;

    // Mark token as used
    sqlx::query("UPDATE invitations SET accepted_at = $1 WHERE id = $2")
        .bind(&now_str)
        .bind(&invitation_id)
        .execute(&state.pool)
        .await?;

    info!(user_id = %user_id, "Password reset completed");

    Ok(Json(MessageResponse {
        message: "Password has been reset".to_string(),
    }))
}

/// GET /api/v1/auth/verify-email/:token
pub async fn verify_email(
    State(state): State<PlatformApiState>,
    Path(token): Path<String>,
) -> Result<Json<MessageResponse>, AuthError> {
    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let row = sqlx::query(
        "SELECT id, org_id, expires_at, accepted_at
         FROM invitations
         WHERE token = $1 AND role = 'email_verification'",
    )
    .bind(&token)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AuthError::BadRequest("Invalid verification token".to_string()))?;

    let accepted_at: Option<String> = row.try_get("accepted_at").ok().flatten();
    if accepted_at.is_some() {
        return Err(AuthError::BadRequest("Email already verified".to_string()));
    }

    let expires_at: String = row.get("expires_at");
    if expires_at < now_str {
        return Err(AuthError::BadRequest(
            "Verification token has expired".to_string(),
        ));
    }

    let user_id: String = row.get("org_id");
    let invitation_id: String = row.get("id");

    // Mark user email as verified
    sqlx::query("UPDATE users SET email_verified = 1, updated_at = $1 WHERE id = $2")
        .bind(&now_str)
        .bind(&user_id)
        .execute(&state.pool)
        .await?;

    // Mark invitation as accepted
    sqlx::query("UPDATE invitations SET accepted_at = $1 WHERE id = $2")
        .bind(&now_str)
        .bind(&invitation_id)
        .execute(&state.pool)
        .await?;

    info!(user_id = %user_id, "Email verified");

    Ok(Json(MessageResponse {
        message: "Email verified successfully".to_string(),
    }))
}
