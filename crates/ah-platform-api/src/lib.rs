// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Agent Harbor Platform API
//!
//! REST endpoints for platform authentication (sign-up, sign-in, token refresh,
//! password reset, email verification), organization management, team management,
//! and invitations.

pub mod auth_handlers;
pub mod executor_catalog;
pub mod executor_handlers;
pub mod executor_metering;
pub mod invitation_handlers;
pub mod middleware;
pub mod org_handlers;
pub mod team_handlers;

use ah_auth::{JwtConfig, RateLimiter};
use std::sync::{Arc, Mutex};

/// Shared state for all platform API handlers.
#[derive(Clone)]
pub struct PlatformApiState {
    /// Database connection pool (sqlx AnyPool).
    pub pool: sqlx::AnyPool,
    /// JWT configuration (secret, TTLs).
    pub jwt_config: JwtConfig,
    /// Auth-endpoint rate limiter.
    pub rate_limiter: Arc<Mutex<RateLimiter>>,
}

/// Build the platform auth router.
///
/// Mounts all `/api/v1/auth/*` endpoints under a nested Axum router.
pub fn auth_router() -> axum::Router<PlatformApiState> {
    use axum::routing::{get, post};

    axum::Router::new()
        .route("/api/v1/auth/signup", post(auth_handlers::signup))
        .route("/api/v1/auth/signin", post(auth_handlers::signin))
        .route("/api/v1/auth/refresh", post(auth_handlers::refresh))
        .route(
            "/api/v1/auth/reset-password",
            post(auth_handlers::reset_password),
        )
        .route(
            "/api/v1/auth/reset-password/confirm",
            post(auth_handlers::reset_password_confirm),
        )
        .route(
            "/api/v1/auth/verify-email/:token",
            get(auth_handlers::verify_email),
        )
        .route(
            "/api/v1/auth/invite/:token/accept",
            post(invitation_handlers::accept_invitation),
        )
}

/// Build the platform organization router.
///
/// Mounts all `/api/v1/orgs/*` endpoints under a nested Axum router.
pub fn org_router() -> axum::Router<PlatformApiState> {
    use axum::routing::{delete, get, patch, post};

    axum::Router::new()
        .route("/api/v1/orgs", post(org_handlers::create_org))
        .route(
            "/api/v1/orgs/:org_id",
            get(org_handlers::get_org_settings)
                .patch(org_handlers::update_org_settings)
                .delete(org_handlers::delete_org),
        )
        .route(
            "/api/v1/orgs/:org_id/invitations",
            post(org_handlers::invite_members),
        )
}

/// Build the team management router.
///
/// Mounts team and invitation endpoints under `/api/v1/orgs/:org_id/`.
pub fn team_router() -> axum::Router<PlatformApiState> {
    use axum::routing::{delete, get, patch};

    axum::Router::new()
        .route(
            "/api/v1/orgs/:org_id/members",
            get(team_handlers::list_members),
        )
        .route(
            "/api/v1/orgs/:org_id/members/:user_id",
            patch(team_handlers::change_member_role).delete(team_handlers::remove_member),
        )
        .route(
            "/api/v1/orgs/:org_id/team/invitations",
            get(invitation_handlers::list_invitations).post(invitation_handlers::send_invitation),
        )
        .route(
            "/api/v1/orgs/:org_id/team/invitations/:invitation_id",
            delete(invitation_handlers::revoke_invitation),
        )
}

/// Build the managed executor router.
///
/// Mounts executor CRUD under `/api/v1/orgs/:org_id/executors` and the
/// public catalog at `/api/v1/catalog/machine-classes`.
pub fn executor_router() -> axum::Router<PlatformApiState> {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route(
            "/api/v1/orgs/:org_id/executors",
            get(executor_handlers::list_executors).post(executor_handlers::provision_executor),
        )
        .route(
            "/api/v1/orgs/:org_id/executors/:executor_id/stop",
            post(executor_handlers::stop_executor),
        )
        .route(
            "/api/v1/orgs/:org_id/executors/:executor_id/start",
            post(executor_handlers::start_executor),
        )
        .route(
            "/api/v1/orgs/:org_id/executors/:executor_id",
            delete(executor_handlers::terminate_executor),
        )
        .route(
            "/api/v1/catalog/machine-classes",
            get(executor_handlers::catalog_machine_classes),
        )
}

/// Build the complete platform API router (auth + org + team + executor routes).
pub fn platform_router() -> axum::Router<PlatformApiState> {
    auth_router().merge(org_router()).merge(team_router()).merge(executor_router())
}
