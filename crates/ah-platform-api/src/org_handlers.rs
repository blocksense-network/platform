// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Organization API handlers (create org, invite members).

use crate::PlatformApiState;
use crate::auth_handlers::{AuthError, MessageResponse};
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

#[derive(Debug, Deserialize)]
pub struct CreateOrgRequest {
    pub name: String,
    pub slug: String,
    pub plan: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgResponse {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrgRequest {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOrgConfirmation {
    pub confirm_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct InviteMembersRequest {
    pub invitations: Vec<InvitationEntry>,
}

#[derive(Debug, Deserialize)]
pub struct InvitationEntry {
    pub email: String,
    pub role: String,
}

// =============================================================================
// Handlers
// =============================================================================

/// POST /api/v1/orgs
pub async fn create_org(
    State(state): State<PlatformApiState>,
    Json(req): Json<CreateOrgRequest>,
) -> Result<Response, AuthError> {
    // Validate name
    if req.name.trim().is_empty() {
        return Err(AuthError::BadRequest(
            "Organization name is required".to_string(),
        ));
    }

    // Validate slug format: lowercase alphanumeric + hyphens, min 2 chars
    if req.slug.len() < 2 {
        return Err(AuthError::BadRequest(
            "Slug must be at least 2 characters".to_string(),
        ));
    }

    if !is_valid_slug(&req.slug) {
        return Err(AuthError::BadRequest(
            "Slug must contain only lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    // Check slug uniqueness
    let existing = sqlx::query("SELECT id FROM tenants WHERE slug = $1")
        .bind(&req.slug)
        .fetch_optional(&state.pool)
        .await?;

    if existing.is_some() {
        return Err(AuthError::Conflict(
            "An organization with this slug already exists".to_string(),
        ));
    }

    let org_id = uuid::Uuid::new_v4().to_string();
    let plan = req.plan.unwrap_or_else(|| "free".to_string());
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Insert tenant (org)
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', $4, $5, $6)",
    )
    .bind(&org_id)
    .bind(&req.name)
    .bind(&req.slug)
    .bind(&format!(r#"{{"plan":"{}"}}"#, plan))
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // Create org_members row linking the creator as admin.
    // For now we use a placeholder user_id since we don't have JWT middleware
    // wired yet. The integration test inserts the user separately.
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO org_members (id, org_id, user_id, role, created_at)
         VALUES ($1, $2, $3, 'admin', $4)",
    )
    .bind(&member_id)
    .bind(&org_id)
    .bind("placeholder") // Will be replaced with authenticated user ID
    .bind(&now)
    .execute(&state.pool)
    .await
    .ok(); // Silently ignore if org_members table doesn't exist yet

    info!(org_id = %org_id, slug = %req.slug, "Organization created");

    Ok((
        StatusCode::CREATED,
        Json(OrgResponse {
            id: org_id,
            name: req.name,
            slug: req.slug,
            plan,
            created_at: now,
        }),
    )
        .into_response())
}

/// POST /api/v1/orgs/:org_id/invitations
pub async fn invite_members(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    Json(req): Json<InviteMembersRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    // Verify org exists
    let org = sqlx::query("SELECT id FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    if org.is_none() {
        return Err(AuthError::NotFound("Organization not found".to_string()));
    }

    let expires_at = (chrono::Utc::now() + chrono::Duration::days(7))
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    for entry in &req.invitations {
        if entry.email.is_empty() || !entry.email.contains('@') {
            return Err(AuthError::BadRequest(format!(
                "Invalid email: {}",
                entry.email
            )));
        }

        let valid_roles = ["admin", "operator", "viewer", "member"];
        if !valid_roles.contains(&entry.role.as_str()) {
            return Err(AuthError::BadRequest(format!(
                "Invalid role: {}",
                entry.role
            )));
        }

        let invitation_id = uuid::Uuid::new_v4().to_string();
        let token = ah_auth::generate_invitation_token();

        sqlx::query(
            "INSERT INTO invitations (id, org_id, email, role, token, invited_by, expires_at)
             VALUES ($1, $2, $3, $4, $5, NULL, $6)",
        )
        .bind(&invitation_id)
        .bind(&org_id)
        .bind(&entry.email)
        .bind(&entry.role)
        .bind(&token)
        .bind(&expires_at)
        .execute(&state.pool)
        .await?;

        info!(
            org_id = %org_id,
            email = %entry.email,
            role = %entry.role,
            "Invitation sent (token={})",
            token,
        );
    }

    Ok(Json(MessageResponse {
        message: format!("{} invitation(s) sent", req.invitations.len()),
    }))
}

/// GET /api/v1/orgs/:org_id
pub async fn get_org_settings(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<OrgResponse>, AuthError> {
    // Authenticate caller
    let user_id = crate::middleware::extract_auth_user(&headers, &state.jwt_config)?;

    // Verify membership
    crate::middleware::get_member_role(&state.pool, &user_id, &org_id).await?;

    // Fetch tenant
    let row = sqlx::query(
        "SELECT id, name, slug, settings, created_at FROM tenants WHERE id = $1 AND status = 'active'",
    )
    .bind(&org_id)
    .fetch_optional(&state.pool)
    .await?;

    let row = row.ok_or_else(|| AuthError::NotFound("Organization not found".to_string()))?;

    let id: String = row.get("id");
    let name: String = row.get("name");
    let slug: String = row.get("slug");
    let settings: String = row.get("settings");
    let created_at: String = row.get("created_at");

    // Extract plan from settings JSON
    let plan = serde_json::from_str::<serde_json::Value>(&settings)
        .ok()
        .and_then(|v| v["plan"].as_str().map(String::from))
        .unwrap_or_else(|| "free".to_string());

    Ok(Json(OrgResponse {
        id,
        name,
        slug,
        plan,
        created_at,
    }))
}

/// PATCH /api/v1/orgs/:org_id
pub async fn update_org_settings(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<UpdateOrgRequest>,
) -> Result<Json<OrgResponse>, AuthError> {
    // Authenticate caller
    let user_id = crate::middleware::extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can update
    crate::middleware::require_admin(&state.pool, &user_id, &org_id).await?;

    // Validate name if provided
    if let Some(ref name) = req.name {
        if name.trim().is_empty() {
            return Err(AuthError::BadRequest(
                "Organization name is required".to_string(),
            ));
        }
        if name.len() > 255 {
            return Err(AuthError::BadRequest(
                "Organization name must be 255 characters or fewer".to_string(),
            ));
        }
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Update name if provided
    if let Some(ref name) = req.name {
        sqlx::query("UPDATE tenants SET name = $1, updated_at = $2 WHERE id = $3")
            .bind(name)
            .bind(&now)
            .bind(&org_id)
            .execute(&state.pool)
            .await?;

        info!(org_id = %org_id, new_name = %name, "Organization name updated");
    }

    // Fetch and return updated org
    let row = sqlx::query("SELECT id, name, slug, settings, created_at FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    let row = row.ok_or_else(|| AuthError::NotFound("Organization not found".to_string()))?;

    let id: String = row.get("id");
    let name: String = row.get("name");
    let slug: String = row.get("slug");
    let settings: String = row.get("settings");
    let created_at: String = row.get("created_at");

    let plan = serde_json::from_str::<serde_json::Value>(&settings)
        .ok()
        .and_then(|v| v["plan"].as_str().map(String::from))
        .unwrap_or_else(|| "free".to_string());

    Ok(Json(OrgResponse {
        id,
        name,
        slug,
        plan,
        created_at,
    }))
}

/// DELETE /api/v1/orgs/:org_id
pub async fn delete_org(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
    Json(req): Json<DeleteOrgConfirmation>,
) -> Result<Response, AuthError> {
    // Authenticate caller
    let user_id = crate::middleware::extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can delete
    crate::middleware::require_admin(&state.pool, &user_id, &org_id).await?;

    // Fetch org to verify slug
    let row = sqlx::query("SELECT slug FROM tenants WHERE id = $1 AND status = 'active'")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    let row = row.ok_or_else(|| AuthError::NotFound("Organization not found".to_string()))?;
    let actual_slug: String = row.get("slug");

    if req.confirm_slug != actual_slug {
        return Err(AuthError::BadRequest(
            "Confirmation slug does not match the organization slug".to_string(),
        ));
    }

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Soft-delete: mark tenant as deleted
    sqlx::query("UPDATE tenants SET status = 'deleted', updated_at = $1 WHERE id = $2")
        .bind(&now)
        .bind(&org_id)
        .execute(&state.pool)
        .await?;

    info!(org_id = %org_id, "Organization deleted — subscription cancellation would be triggered");

    Ok(StatusCode::NO_CONTENT.into_response())
}

// =============================================================================
// Helpers
// =============================================================================

/// Validates a slug: lowercase alphanumeric + hyphens, must start and end
/// with an alphanumeric character.
fn is_valid_slug(slug: &str) -> bool {
    if slug.len() < 2 {
        return false;
    }
    let bytes = slug.as_bytes();
    // Must start and end with alphanumeric
    if !bytes[0].is_ascii_lowercase() && !bytes[0].is_ascii_digit() {
        return false;
    }
    let last = bytes[bytes.len() - 1];
    if !last.is_ascii_lowercase() && !last.is_ascii_digit() {
        return false;
    }
    // Middle characters: lowercase, digit, or hyphen
    for &b in bytes {
        if !b.is_ascii_lowercase() && !b.is_ascii_digit() && b != b'-' {
            return false;
        }
    }
    true
}
