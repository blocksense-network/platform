// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Invitation API handlers (list, send, revoke, accept invitations).

use crate::PlatformApiState;
use crate::auth_handlers::AuthError;
use crate::middleware::{extract_auth_user, require_admin};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::info;

// =============================================================================
// Request / response types
// =============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    pub id: String,
    pub email: String,
    pub role: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SendInvitationRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgInfoResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/orgs/:org_id/invitations
pub async fn list_invitations(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<InvitationResponse>>, AuthError> {
    let user_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Verify caller is a member
    crate::middleware::get_member_role(&state.pool, &user_id, &org_id).await?;

    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let rows = sqlx::query(
        "SELECT id, email, role, token, expires_at, created_at
         FROM invitations
         WHERE org_id = $1
           AND accepted_at IS NULL
           AND expires_at > $2
           AND role NOT IN ('email_verification', 'password_reset')
         ORDER BY created_at DESC",
    )
    .bind(&org_id)
    .bind(&now_str)
    .fetch_all(&state.pool)
    .await?;

    let invitations: Vec<InvitationResponse> = rows
        .iter()
        .map(|r| InvitationResponse {
            id: r.get("id"),
            email: r.get("email"),
            role: r.get("role"),
            token: r.get("token"),
            expires_at: r.get("expires_at"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(invitations))
}

/// POST /api/v1/orgs/:org_id/invitations
pub async fn send_invitation(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<SendInvitationRequest>,
) -> Result<Response, AuthError> {
    let caller_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can invite
    require_admin(&state.pool, &caller_id, &org_id).await?;

    // Validate email
    if req.email.is_empty() || !req.email.contains('@') {
        return Err(AuthError::BadRequest(format!(
            "Invalid email: {}",
            req.email
        )));
    }

    // Validate role
    let valid_roles = ["admin", "operator", "viewer"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AuthError::BadRequest(format!("Invalid role: {}", req.role)));
    }

    // Check seat limit
    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Get plan seats limit from tenant settings
    let tenant_row = sqlx::query("SELECT settings FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    let tenant_row =
        tenant_row.ok_or_else(|| AuthError::NotFound("Organization not found".to_string()))?;

    let settings_str: Option<String> = tenant_row.try_get("settings").ok().flatten();
    let seats_limit = if let Some(settings) = settings_str {
        // Parse the plan from settings JSON
        let plan = serde_json::from_str::<serde_json::Value>(&settings)
            .ok()
            .and_then(|v| v.get("plan").and_then(|p| p.as_str()).map(String::from))
            .unwrap_or_else(|| "free".to_string());

        match plan.as_str() {
            "free" => Some(2),
            "team" => Some(10),
            "enterprise" => None, // unlimited
            _ => Some(2),
        }
    } else {
        Some(2) // default free plan limit
    };

    if let Some(limit) = seats_limit {
        // Count current members
        let member_count_row =
            sqlx::query("SELECT COUNT(*) as cnt FROM org_members WHERE org_id = $1")
                .bind(&org_id)
                .fetch_one(&state.pool)
                .await?;
        let member_count: i64 = member_count_row.get("cnt");

        // Count pending invitations (not expired, not accepted, not internal types)
        let pending_count_row = sqlx::query(
            "SELECT COUNT(*) as cnt FROM invitations
             WHERE org_id = $1
               AND accepted_at IS NULL
               AND expires_at > $2
               AND role NOT IN ('email_verification', 'password_reset')",
        )
        .bind(&org_id)
        .bind(&now_str)
        .fetch_one(&state.pool)
        .await?;
        let pending_count: i64 = pending_count_row.get("cnt");

        if member_count + pending_count >= limit as i64 {
            return Err(AuthError::Forbidden(
                "Seat limit reached for this plan".to_string(),
            ));
        }
    }

    let invitation_id = uuid::Uuid::new_v4().to_string();
    let token = ah_auth::generate_invitation_token();
    let expires_at = (chrono::Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query(
        "INSERT INTO invitations (id, org_id, email, role, token, invited_by, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&invitation_id)
    .bind(&org_id)
    .bind(&req.email)
    .bind(&req.role)
    .bind(&token)
    .bind(&caller_id)
    .bind(&expires_at)
    .execute(&state.pool)
    .await?;

    info!(
        org_id = %org_id,
        email = %req.email,
        role = %req.role,
        "Invitation email would be sent (token={})",
        token,
    );

    let created_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    Ok((
        StatusCode::CREATED,
        Json(InvitationResponse {
            id: invitation_id,
            email: req.email,
            role: req.role,
            token,
            expires_at,
            created_at,
        }),
    )
        .into_response())
}

/// DELETE /api/v1/orgs/:org_id/invitations/:invitation_id
pub async fn revoke_invitation(
    State(state): State<PlatformApiState>,
    Path((org_id, invitation_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let caller_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can revoke
    require_admin(&state.pool, &caller_id, &org_id).await?;

    // Verify invitation exists and belongs to this org
    let row = sqlx::query("SELECT id FROM invitations WHERE id = $1 AND org_id = $2")
        .bind(&invitation_id)
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    if row.is_none() {
        return Err(AuthError::NotFound("Invitation not found".to_string()));
    }

    sqlx::query("DELETE FROM invitations WHERE id = $1")
        .bind(&invitation_id)
        .execute(&state.pool)
        .await?;

    info!(
        org_id = %org_id,
        invitation_id = %invitation_id,
        "Invitation revoked"
    );

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// POST /api/v1/auth/invite/:token/accept
pub async fn accept_invitation(
    State(state): State<PlatformApiState>,
    Path(token): Path<String>,
) -> Result<Json<OrgInfoResponse>, AuthError> {
    let now_str = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Look up invitation by token
    let row = sqlx::query(
        "SELECT id, org_id, email, role, expires_at, accepted_at
         FROM invitations
         WHERE token = $1
           AND role NOT IN ('email_verification', 'password_reset')",
    )
    .bind(&token)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AuthError::BadRequest("Invalid invitation token".to_string()))?;

    // Check if already accepted
    let accepted_at: Option<String> = row.try_get("accepted_at").ok().flatten();
    if accepted_at.is_some() {
        return Err(AuthError::BadRequest(
            "Invitation has already been accepted".to_string(),
        ));
    }

    // Check if expired
    let expires_at: String = row.get("expires_at");
    if expires_at < now_str {
        return Err(AuthError::BadRequest("Invitation has expired".to_string()));
    }

    let invitation_id: String = row.get("id");
    let org_id: String = row.get("org_id");
    let email: String = row.get("email");
    let role: String = row.get("role");

    // Check if user exists by email
    let user_row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.pool)
        .await?;

    let user_row = user_row.ok_or_else(|| {
        AuthError::BadRequest(
            "Please create an account first, then accept the invitation".to_string(),
        )
    })?;

    let user_id: String = user_row.get("id");

    // Add user to org_members
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO org_members (id, org_id, user_id, role, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&member_id)
    .bind(&org_id)
    .bind(&user_id)
    .bind(&role)
    .bind(&now_str)
    .execute(&state.pool)
    .await?;

    // Mark invitation as accepted
    sqlx::query("UPDATE invitations SET accepted_at = $1 WHERE id = $2")
        .bind(&now_str)
        .bind(&invitation_id)
        .execute(&state.pool)
        .await?;

    info!(
        org_id = %org_id,
        user_id = %user_id,
        email = %email,
        "Invitation accepted"
    );

    // Return org details
    let org_row = sqlx::query("SELECT id, name, slug FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(OrgInfoResponse {
        id: org_row.get("id"),
        name: org_row.get("name"),
        slug: org_row.get("slug"),
    }))
}
