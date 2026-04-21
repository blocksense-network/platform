// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Team management API handlers (list members, change role, remove member).

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
pub struct MemberResponse {
    pub user_id: String,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/orgs/:org_id/members
pub async fn list_members(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<MemberResponse>>, AuthError> {
    let user_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Verify caller is a member of the org
    crate::middleware::get_member_role(&state.pool, &user_id, &org_id).await?;

    let rows = sqlx::query(
        "SELECT om.user_id, u.email, u.name, om.role, om.created_at as joined_at
         FROM org_members om
         JOIN users u ON u.id = om.user_id
         WHERE om.org_id = $1
         ORDER BY om.created_at ASC",
    )
    .bind(&org_id)
    .fetch_all(&state.pool)
    .await?;

    let members: Vec<MemberResponse> = rows
        .iter()
        .map(|r| MemberResponse {
            user_id: r.get("user_id"),
            email: r.get("email"),
            name: r.try_get("name").ok().flatten(),
            role: r.get("role"),
            joined_at: r.get("joined_at"),
        })
        .collect();

    Ok(Json(members))
}

/// PATCH /api/v1/orgs/:org_id/members/:user_id
pub async fn change_member_role(
    State(state): State<PlatformApiState>,
    Path((org_id, target_user_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(req): Json<ChangeRoleRequest>,
) -> Result<Json<MemberResponse>, AuthError> {
    let caller_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can change roles
    require_admin(&state.pool, &caller_id, &org_id).await?;

    // Cannot change own role
    if caller_id == target_user_id {
        return Err(AuthError::BadRequest(
            "Cannot change your own role".to_string(),
        ));
    }

    // Validate role
    let valid_roles = ["admin", "operator", "viewer"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AuthError::BadRequest(format!(
            "Invalid role: {}. Must be one of: admin, operator, viewer",
            req.role
        )));
    }

    // Verify target is a member
    crate::middleware::get_member_role(&state.pool, &target_user_id, &org_id).await?;

    // Update role
    sqlx::query("UPDATE org_members SET role = $1 WHERE org_id = $2 AND user_id = $3")
        .bind(&req.role)
        .bind(&org_id)
        .bind(&target_user_id)
        .execute(&state.pool)
        .await?;

    info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        new_role = %req.role,
        "Member role changed"
    );

    // Fetch updated member info
    let row = sqlx::query(
        "SELECT om.user_id, u.email, u.name, om.role, om.created_at as joined_at
         FROM org_members om
         JOIN users u ON u.id = om.user_id
         WHERE om.org_id = $1 AND om.user_id = $2",
    )
    .bind(&org_id)
    .bind(&target_user_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(MemberResponse {
        user_id: row.get("user_id"),
        email: row.get("email"),
        name: row.try_get("name").ok().flatten(),
        role: row.get("role"),
        joined_at: row.get("joined_at"),
    }))
}

/// DELETE /api/v1/orgs/:org_id/members/:user_id
pub async fn remove_member(
    State(state): State<PlatformApiState>,
    Path((org_id, target_user_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response, AuthError> {
    let caller_id = extract_auth_user(&headers, &state.jwt_config)?;

    // Only admins can remove members
    require_admin(&state.pool, &caller_id, &org_id).await?;

    // Cannot remove self
    if caller_id == target_user_id {
        return Err(AuthError::BadRequest(
            "Cannot remove yourself from the organization".to_string(),
        ));
    }

    // Verify target is a member
    crate::middleware::get_member_role(&state.pool, &target_user_id, &org_id).await?;

    sqlx::query("DELETE FROM org_members WHERE org_id = $1 AND user_id = $2")
        .bind(&org_id)
        .bind(&target_user_id)
        .execute(&state.pool)
        .await?;

    info!(
        org_id = %org_id,
        target_user_id = %target_user_id,
        "Member removed from organization"
    );

    Ok(StatusCode::NO_CONTENT.into_response())
}
