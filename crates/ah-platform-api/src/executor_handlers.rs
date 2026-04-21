// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Managed executor API handlers (provision, list, start, stop, terminate).

use crate::PlatformApiState;
use crate::auth_handlers::AuthError;
use crate::executor_catalog;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::info;

// =============================================================================
// Request / response types
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisionExecutorRequest {
    pub machine_class: String,
    pub os: String,
    pub region: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorResponse {
    pub id: String,
    pub org_id: String,
    pub machine_class: String,
    pub os: String,
    pub region: String,
    pub name: Option<String>,
    pub status: String,
    pub hourly_rate_cents: u32,
    pub created_at: String,
    pub provisioned_at: Option<String>,
    pub terminated_at: Option<String>,
}

// =============================================================================
// Handlers
// =============================================================================

/// GET /api/v1/orgs/:org_id/executors
pub async fn list_executors(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
) -> Result<Json<Vec<ExecutorResponse>>, AuthError> {
    // Verify org exists
    let org = sqlx::query("SELECT id FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    if org.is_none() {
        return Err(AuthError::NotFound("Organization not found".to_string()));
    }

    let rows = sqlx::query(
        "SELECT id, org_id, machine_class, os, region, name, status, hourly_rate_cents, \
         created_at, provisioned_at, terminated_at \
         FROM managed_executors WHERE org_id = $1 ORDER BY created_at DESC",
    )
    .bind(&org_id)
    .fetch_all(&state.pool)
    .await?;

    let executors: Vec<ExecutorResponse> = rows
        .iter()
        .map(|row| ExecutorResponse {
            id: row.get("id"),
            org_id: row.get("org_id"),
            machine_class: row.get("machine_class"),
            os: row.get("os"),
            region: row.get("region"),
            name: row.get("name"),
            status: row.get("status"),
            hourly_rate_cents: row.get::<i32, _>("hourly_rate_cents") as u32,
            created_at: row.get("created_at"),
            provisioned_at: row.get("provisioned_at"),
            terminated_at: row.get("terminated_at"),
        })
        .collect();

    Ok(Json(executors))
}

/// POST /api/v1/orgs/:org_id/executors
pub async fn provision_executor(
    State(state): State<PlatformApiState>,
    Path(org_id): Path<String>,
    Json(req): Json<ProvisionExecutorRequest>,
) -> Result<Response, AuthError> {
    // Verify org exists and get plan
    let org_row = sqlx::query("SELECT id, settings FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_optional(&state.pool)
        .await?;

    let org_row = match org_row {
        Some(r) => r,
        None => return Err(AuthError::NotFound("Organization not found".to_string())),
    };

    // Check plan — free plan cannot provision managed executors
    let settings: Option<String> = org_row.get("settings");
    let plan = settings
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("plan").and_then(|p| p.as_str().map(|s| s.to_string())))
        .unwrap_or_else(|| "free".to_string());

    if plan == "free" {
        return Err(AuthError::Forbidden(
            "Managed executors are not available on the free plan. Please upgrade.".to_string(),
        ));
    }

    // Validate machine class
    let machine_class =
        executor_catalog::get_machine_class(&req.machine_class).ok_or_else(|| {
            AuthError::BadRequest(format!("Unknown machine class: {}", req.machine_class))
        })?;

    // Validate OS is available for this class
    if !machine_class.available_os.contains(&req.os.as_str()) {
        return Err(AuthError::BadRequest(format!(
            "OS '{}' is not available for machine class '{}'. Available: {:?}",
            req.os, req.machine_class, machine_class.available_os
        )));
    }

    // Validate region is available for this class
    if !machine_class.available_regions.contains(&req.region.as_str()) {
        return Err(AuthError::BadRequest(format!(
            "Region '{}' is not available for machine class '{}'. Available: {:?}",
            req.region, req.machine_class, machine_class.available_regions
        )));
    }

    let executor_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    sqlx::query(
        "INSERT INTO managed_executors \
         (id, org_id, machine_class, os, region, name, status, hourly_rate_cents, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, 'provisioning', $7, $8, $9)",
    )
    .bind(&executor_id)
    .bind(&org_id)
    .bind(machine_class.id)
    .bind(&req.os)
    .bind(&req.region)
    .bind(&req.name)
    .bind(machine_class.hourly_rate_cents as i32)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    info!(
        executor_id = %executor_id,
        org_id = %org_id,
        machine_class = machine_class.id,
        "Managed executor provisioning would be triggered"
    );

    Ok((
        StatusCode::CREATED,
        Json(ExecutorResponse {
            id: executor_id,
            org_id,
            machine_class: machine_class.id.to_string(),
            os: req.os,
            region: req.region,
            name: req.name,
            status: "provisioning".to_string(),
            hourly_rate_cents: machine_class.hourly_rate_cents,
            created_at: now,
            provisioned_at: None,
            terminated_at: None,
        }),
    )
        .into_response())
}

/// POST /api/v1/orgs/:org_id/executors/:executor_id/stop
pub async fn stop_executor(
    State(state): State<PlatformApiState>,
    Path((org_id, executor_id)): Path<(String, String)>,
) -> Result<Json<ExecutorResponse>, AuthError> {
    update_executor_status(&state.pool, &org_id, &executor_id, "stopped", false).await
}

/// POST /api/v1/orgs/:org_id/executors/:executor_id/start
pub async fn start_executor(
    State(state): State<PlatformApiState>,
    Path((org_id, executor_id)): Path<(String, String)>,
) -> Result<Json<ExecutorResponse>, AuthError> {
    update_executor_status(&state.pool, &org_id, &executor_id, "running", false).await
}

/// DELETE /api/v1/orgs/:org_id/executors/:executor_id
pub async fn terminate_executor(
    State(state): State<PlatformApiState>,
    Path((org_id, executor_id)): Path<(String, String)>,
) -> Result<Json<ExecutorResponse>, AuthError> {
    update_executor_status(&state.pool, &org_id, &executor_id, "terminated", true).await
}

/// GET /api/v1/catalog/machine-classes
pub async fn catalog_machine_classes() -> Json<Vec<&'static executor_catalog::MachineClass>> {
    Json(executor_catalog::list_machine_classes().iter().collect())
}

// =============================================================================
// Helpers
// =============================================================================

async fn update_executor_status(
    pool: &sqlx::AnyPool,
    org_id: &str,
    executor_id: &str,
    new_status: &str,
    set_terminated_at: bool,
) -> Result<Json<ExecutorResponse>, AuthError> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Verify executor exists and belongs to org
    let row = sqlx::query("SELECT id FROM managed_executors WHERE id = $1 AND org_id = $2")
        .bind(executor_id)
        .bind(org_id)
        .fetch_optional(pool)
        .await?;

    if row.is_none() {
        return Err(AuthError::NotFound("Executor not found".to_string()));
    }

    if set_terminated_at {
        sqlx::query(
            "UPDATE managed_executors SET status = $1, terminated_at = $2, updated_at = $3 \
             WHERE id = $4 AND org_id = $5",
        )
        .bind(new_status)
        .bind(&now)
        .bind(&now)
        .bind(executor_id)
        .bind(org_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE managed_executors SET status = $1, updated_at = $2 \
             WHERE id = $3 AND org_id = $4",
        )
        .bind(new_status)
        .bind(&now)
        .bind(executor_id)
        .bind(org_id)
        .execute(pool)
        .await?;
    }

    // Fetch updated record
    let updated = sqlx::query(
        "SELECT id, org_id, machine_class, os, region, name, status, hourly_rate_cents, \
         created_at, provisioned_at, terminated_at \
         FROM managed_executors WHERE id = $1",
    )
    .bind(executor_id)
    .fetch_one(pool)
    .await?;

    Ok(Json(ExecutorResponse {
        id: updated.get("id"),
        org_id: updated.get("org_id"),
        machine_class: updated.get("machine_class"),
        os: updated.get("os"),
        region: updated.get("region"),
        name: updated.get("name"),
        status: updated.get("status"),
        hourly_rate_cents: updated.get::<i32, _>("hourly_rate_cents") as u32,
        created_at: updated.get("created_at"),
        provisioned_at: updated.get("provisioned_at"),
        terminated_at: updated.get("terminated_at"),
    }))
}
