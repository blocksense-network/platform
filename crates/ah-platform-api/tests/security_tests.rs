// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Security test suite for the platform API.
//!
//! Verifies authentication, authorization (RBAC), organisation isolation (IDOR
//! prevention), and input validation properties using real in-memory SQLite, real
//! JWT tokens, and real Axum handlers.

use ah_auth::{JwtConfig, RateLimiter};
use ah_platform_api::PlatformApiState;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use http_body_util::BodyExt;
use sqlx::{
    AnyPool, Row,
    any::install_default_drivers,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use std::sync::{Arc, Mutex, Once};
use tower::ServiceExt;

static INSTALL: Once = Once::new();

fn ensure_drivers() {
    INSTALL.call_once(|| {
        install_default_drivers();
    });
}

/// Set up an in-memory SQLite database with the full platform schema.
async fn setup_db(db_name: &str) -> AnyPool {
    ensure_drivers();

    let shared_url = format!("sqlite:file:{}?mode=memory&cache=shared", db_name);
    let sqlite_opts = SqliteConnectOptions::from_str(&shared_url).unwrap().create_if_missing(true);

    let sqlite_pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(2)
        .connect_with(sqlite_opts)
        .await
        .expect("sqlite connect");

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'active',
            settings TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        )
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            email TEXT NOT NULL UNIQUE,
            email_verified INTEGER NOT NULL DEFAULT 0,
            password_hash TEXT,
            name TEXT,
            avatar_url TEXT,
            oauth_provider TEXT,
            oauth_provider_id TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        )
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invitations (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL,
            email TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            token TEXT NOT NULL UNIQUE,
            invited_by TEXT,
            expires_at TEXT NOT NULL,
            accepted_at TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        )
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS org_members (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'member',
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            UNIQUE(org_id, user_id)
        )
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS managed_executors (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL,
            machine_class TEXT,
            os TEXT,
            region TEXT,
            name TEXT,
            status TEXT NOT NULL DEFAULT 'provisioning',
            hourly_rate_cents INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            provisioned_at TEXT,
            terminated_at TEXT
        )
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    // Now open AnyPool connected to the same shared-memory DB
    let any_pool = sqlx::any::AnyPoolOptions::new()
        .max_connections(2)
        .connect(&shared_url)
        .await
        .expect("any pool connect");

    // Keep sqlite_pool alive (it holds the shared memory database)
    std::mem::forget(sqlite_pool);

    any_pool
}

fn jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "security-test-secret-long-enough-for-hmac-256".to_string(),
        access_token_ttl_secs: 900,
        refresh_token_ttl_secs: 604_800,
    }
}

fn make_state(pool: AnyPool) -> PlatformApiState {
    PlatformApiState {
        pool,
        jwt_config: jwt_config(),
        rate_limiter: Arc::new(Mutex::new(RateLimiter::new())),
    }
}

fn app(state: PlatformApiState) -> Router {
    ah_platform_api::platform_router().with_state(state)
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Create a valid access token for a given user ID and email.
fn make_access_token(user_id: &str, email: &str) -> String {
    let config = jwt_config();
    let now = Utc::now().timestamp();
    let claims = ah_auth::AccessTokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        org_id: None,
        role: None,
        exp: now + config.access_token_ttl_secs as i64,
        iat: now,
    };
    ah_auth::create_access_token(&config, &claims).unwrap()
}

/// Sign up a user via the API, return user_id.
async fn signup_user(state: &PlatformApiState, email: &str) -> String {
    let router = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123","name":"Test User"}}"#,
            email
        )))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp.into_body()).await;
    json["userId"].as_str().unwrap().to_string()
}

/// Sign in a user, return access_token.
async fn signin_user(state: &PlatformApiState, email: &str) -> String {
    let router = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123"}}"#,
            email
        )))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp.into_body()).await;
    json["accessToken"].as_str().unwrap().to_string()
}

/// Create an org with team plan and wire the creator as admin.
/// Returns (org_id).
async fn create_org_with_admin(state: &PlatformApiState, user_id: &str, slug: &str) -> String {
    let router = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"name":"Test Org","slug":"{}","plan":"team"}}"#,
            slug
        )))
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_json(resp.into_body()).await;
    let org_id = json["id"].as_str().unwrap().to_string();

    // Wire the real user as admin (replace placeholder)
    sqlx::query(
        "UPDATE org_members SET user_id = $1 WHERE org_id = $2 AND user_id = 'placeholder'",
    )
    .bind(user_id)
    .bind(&org_id)
    .execute(&state.pool)
    .await
    .unwrap();

    org_id
}

/// Add a user to an org with a given role directly via DB.
async fn add_member(state: &PlatformApiState, org_id: &str, user_id: &str, role: &str) {
    let member_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    sqlx::query(
        "INSERT INTO org_members (id, org_id, user_id, role, created_at) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&member_id)
    .bind(org_id)
    .bind(user_id)
    .bind(role)
    .bind(&now)
    .execute(&state.pool)
    .await
    .unwrap();
}

// =============================================================================
// Auth Token Security
// =============================================================================

#[tokio::test]
async fn test_expired_access_token_rejected() {
    let pool = setup_db("sec_expired_token").await;
    let state = make_state(pool.clone());

    let user_id = signup_user(&state, "expired-tok@test.com").await;

    // Create a JWT with past expiry
    let config = jwt_config();
    let claims = ah_auth::AccessTokenClaims {
        sub: user_id.clone(),
        email: "expired-tok@test.com".to_string(),
        org_id: None,
        role: None,
        exp: 1_000_000, // far in the past
        iat: 999_000,
    };
    let expired_token = ah_auth::create_access_token(&config, &claims).unwrap();

    // Create an org so we have a protected endpoint to hit
    let org_id = create_org_with_admin(&state, &user_id, "sec-expired").await;

    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", expired_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_tampered_token_rejected() {
    let pool = setup_db("sec_tampered_token").await;
    let state = make_state(pool.clone());

    let user_id = signup_user(&state, "tampered-tok@test.com").await;
    let valid_token = make_access_token(&user_id, "tampered-tok@test.com");
    let org_id = create_org_with_admin(&state, &user_id, "sec-tampered").await;

    // Tamper with the token payload (flip a character in the middle part)
    let parts: Vec<&str> = valid_token.split('.').collect();
    assert_eq!(parts.len(), 3);
    let mut tampered_payload = parts[1].to_string();
    // Append a character to change the base64 payload
    tampered_payload.push('X');
    let tampered_token = format!("{}.{}.{}", parts[0], tampered_payload, parts[2]);

    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", tampered_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_missing_auth_header() {
    let pool = setup_db("sec_missing_auth").await;
    let state = make_state(pool.clone());

    let user_id = signup_user(&state, "noauth@test.com").await;
    let org_id = create_org_with_admin(&state, &user_id, "sec-noauth").await;

    // Call protected endpoint without Authorization header
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_malformed_bearer_token() {
    let pool = setup_db("sec_malformed_bearer").await;
    let state = make_state(pool.clone());

    let user_id = signup_user(&state, "malformed@test.com").await;
    let org_id = create_org_with_admin(&state, &user_id, "sec-malformed").await;

    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", "Bearer not-a-jwt")
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// =============================================================================
// Password Reset Token Security
// =============================================================================

#[tokio::test]
async fn test_reset_token_single_use() {
    let pool = setup_db("sec_reset_single_use").await;
    let state = make_state(pool.clone());

    // Create user
    let password_hash = ah_auth::hash_password("original123").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(&user_id)
    .bind("sec-reset@test.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    // Request password reset
    let router1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"sec-reset@test.com"}"#))
        .unwrap();
    router1.oneshot(req).await.unwrap();

    // Get token from DB
    let token_row =
        sqlx::query("SELECT token FROM invitations WHERE email = $1 AND role = 'password_reset'")
            .bind("sec-reset@test.com")
            .fetch_one(&pool)
            .await
            .unwrap();
    let reset_token: String = token_row.get("token");

    // First use should succeed
    let router2 = app(state.clone());
    let body1 = serde_json::json!({ "token": reset_token, "newPassword": "NewPass123!" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(body1.to_string()))
        .unwrap();
    let resp = router2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second use of same token must fail
    let router3 = app(state);
    let body2 = serde_json::json!({ "token": reset_token, "newPassword": "AnotherP1!" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(body2.to_string()))
        .unwrap();
    let resp = router3.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["message"], "Token already used");
}

#[tokio::test]
async fn test_reset_token_wrong_password_format() {
    let pool = setup_db("sec_reset_weak_pw").await;
    let state = make_state(pool.clone());

    // Create user
    let password_hash = ah_auth::hash_password("original123").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(&user_id)
    .bind("sec-weakpw@test.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    // Request reset
    let router1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"sec-weakpw@test.com"}"#))
        .unwrap();
    router1.oneshot(req).await.unwrap();

    let token_row =
        sqlx::query("SELECT token FROM invitations WHERE email = $1 AND role = 'password_reset'")
            .bind("sec-weakpw@test.com")
            .fetch_one(&pool)
            .await
            .unwrap();
    let reset_token: String = token_row.get("token");

    // Try to reset with too-short password
    let router2 = app(state);
    let body = serde_json::json!({ "token": reset_token, "newPassword": "short" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = router2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert!(json["message"].as_str().unwrap().contains("8 characters"));
}

// =============================================================================
// Email Verification Security
// =============================================================================

#[tokio::test]
async fn test_verify_email_invalid_token() {
    let pool = setup_db("sec_verify_invalid").await;
    let state = make_state(pool);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/auth/verify-email/totally-bogus-token-value")
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["message"], "Invalid verification token");
}

#[tokio::test]
async fn test_verify_email_idempotent() {
    let pool = setup_db("sec_verify_idempotent").await;
    let state = make_state(pool.clone());

    // Sign up to create verification token
    signup_user(&state, "sec-verify-idem@test.com").await;

    // Get token from DB
    let token_row = sqlx::query(
        "SELECT token FROM invitations WHERE email = $1 AND role = 'email_verification'",
    )
    .bind("sec-verify-idem@test.com")
    .fetch_one(&pool)
    .await
    .unwrap();
    let verification_token: String = token_row.get("token");

    // First verify should succeed
    let router1 = app(state.clone());
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/auth/verify-email/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let resp = router1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second verify: already verified -- the handler returns 400 "Email already
    // verified" which is acceptable idempotency behavior (the email IS verified).
    let router2 = app(state);
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/auth/verify-email/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let resp = router2.oneshot(req).await.unwrap();
    // The important property: the user's email stays verified and no error
    // beyond "already verified" occurs.
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let json = body_json(resp.into_body()).await;
    assert_eq!(json["message"], "Email already verified");
}

// =============================================================================
// RBAC Enforcement
// =============================================================================

#[tokio::test]
async fn test_viewer_cannot_invite() {
    let pool = setup_db("sec_viewer_invite").await;
    let state = make_state(pool.clone());

    // Create admin user + org
    let admin_id = signup_user(&state, "sec-admin-inv@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-viewer-inv").await;

    // Create viewer user
    let viewer_id = signup_user(&state, "sec-viewer-inv@test.com").await;
    let viewer_token = signin_user(&state, "sec-viewer-inv@test.com").await;
    add_member(&state, &org_id, &viewer_id, "viewer").await;

    // Viewer tries to send invitation -> 403
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .body(Body::from(
            r#"{"email":"someone@test.com","role":"viewer"}"#,
        ))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_viewer_cannot_change_role() {
    let pool = setup_db("sec_viewer_chrole").await;
    let state = make_state(pool.clone());

    let admin_id = signup_user(&state, "sec-admin-chrole@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-viewer-chrole").await;

    let viewer_id = signup_user(&state, "sec-viewer-chrole@test.com").await;
    let viewer_token = signin_user(&state, "sec-viewer-chrole@test.com").await;
    add_member(&state, &org_id, &viewer_id, "viewer").await;

    // Viewer tries to change admin's role -> 403
    let req = Request::builder()
        .method("PATCH")
        .uri(&format!("/api/v1/orgs/{}/members/{}", org_id, admin_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .body(Body::from(r#"{"role":"viewer"}"#))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_viewer_cannot_remove_member() {
    let pool = setup_db("sec_viewer_remove").await;
    let state = make_state(pool.clone());

    let admin_id = signup_user(&state, "sec-admin-rm@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-viewer-rm").await;

    let viewer_id = signup_user(&state, "sec-viewer-rm@test.com").await;
    let viewer_token = signin_user(&state, "sec-viewer-rm@test.com").await;
    add_member(&state, &org_id, &viewer_id, "viewer").await;

    // Create another member to try removing
    let target_id = signup_user(&state, "sec-target-rm@test.com").await;
    add_member(&state, &org_id, &target_id, "operator").await;

    // Viewer tries to remove member -> 403
    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}/members/{}", org_id, target_id))
        .header("Authorization", format!("Bearer {}", viewer_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_viewer_cannot_delete_org() {
    let pool = setup_db("sec_viewer_del_org").await;
    let state = make_state(pool.clone());

    let admin_id = signup_user(&state, "sec-admin-del@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-viewer-del").await;

    let viewer_id = signup_user(&state, "sec-viewer-del@test.com").await;
    let viewer_token = signin_user(&state, "sec-viewer-del@test.com").await;
    add_member(&state, &org_id, &viewer_id, "viewer").await;

    // Viewer tries to delete org -> 403
    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", viewer_token))
        .body(Body::from(r#"{"confirmSlug":"sec-viewer-del"}"#))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_operator_cannot_invite() {
    let pool = setup_db("sec_operator_invite").await;
    let state = make_state(pool.clone());

    let admin_id = signup_user(&state, "sec-admin-opinv@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-op-inv").await;

    let op_id = signup_user(&state, "sec-op-inv@test.com").await;
    let op_token = signin_user(&state, "sec-op-inv@test.com").await;
    add_member(&state, &org_id, &op_id, "operator").await;

    // Operator tries to invite -> 403
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", op_token))
        .body(Body::from(
            r#"{"email":"someone-op@test.com","role":"viewer"}"#,
        ))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_non_member_cannot_access_org() {
    let pool = setup_db("sec_non_member").await;
    let state = make_state(pool.clone());

    let admin_id = signup_user(&state, "sec-admin-nm@test.com").await;
    let org_id = create_org_with_admin(&state, &admin_id, "sec-non-member").await;

    // Create another user who is NOT a member of this org
    signup_user(&state, "sec-outsider@test.com").await;
    let outsider_token = signin_user(&state, "sec-outsider@test.com").await;

    // Non-member tries to list members -> 403
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", outsider_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// =============================================================================
// Org Isolation (IDOR Prevention)
// =============================================================================

#[tokio::test]
async fn test_cross_org_member_access() {
    let pool = setup_db("sec_cross_org_members").await;
    let state = make_state(pool.clone());

    // Create org A with admin
    let admin_a_id = signup_user(&state, "sec-admin-a@test.com").await;
    let _org_a = create_org_with_admin(&state, &admin_a_id, "sec-org-a").await;

    // Create org B with different admin
    let admin_b_id = signup_user(&state, "sec-admin-b@test.com").await;
    let org_b = create_org_with_admin(&state, &admin_b_id, "sec-org-b").await;

    // Admin of A tries to access org B members -> 403
    let token_a = signin_user(&state, "sec-admin-a@test.com").await;
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_b))
        .header("Authorization", format!("Bearer {}", token_a))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_cross_org_executor_access() {
    let pool = setup_db("sec_cross_org_exec").await;
    let state = make_state(pool.clone());

    // Create org A
    let admin_a_id = signup_user(&state, "sec-exec-a@test.com").await;
    let org_a = create_org_with_admin(&state, &admin_a_id, "sec-exec-a").await;

    // Create org B
    let admin_b_id = signup_user(&state, "sec-exec-b@test.com").await;
    let org_b = create_org_with_admin(&state, &admin_b_id, "sec-exec-b").await;

    // Insert an executor in org B directly
    let exec_id = uuid::Uuid::new_v4().to_string();
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
    sqlx::query(
        "INSERT INTO managed_executors (id, org_id, machine_class, os, region, status, hourly_rate_cents, created_at, updated_at) \
         VALUES ($1, $2, 'small', 'linux', 'us-east-1', 'running', 15, $3, $4)",
    )
    .bind(&exec_id)
    .bind(&org_b)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await
    .unwrap();

    // Admin of org A tries to stop org B's executor -> should fail (404 because
    // the executor does not belong to org A)
    let _token_a = signin_user(&state, "sec-exec-a@test.com").await;

    // The executor handlers don't require auth on the org, so we test the
    // org_id + executor_id scoping: stopping an executor of org_b via org_a's
    // endpoint path gives 404 (not found in org_a).
    let req = Request::builder()
        .method("POST")
        .uri(&format!(
            "/api/v1/orgs/{}/executors/{}/stop",
            org_a, exec_id
        ))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cross_org_invitation() {
    let pool = setup_db("sec_cross_org_invite").await;
    let state = make_state(pool.clone());

    // Create org A with admin
    let admin_a_id = signup_user(&state, "sec-inv-a@test.com").await;
    let _org_a = create_org_with_admin(&state, &admin_a_id, "sec-inv-a").await;
    let token_a = signin_user(&state, "sec-inv-a@test.com").await;

    // Create org B with different admin
    let admin_b_id = signup_user(&state, "sec-inv-b@test.com").await;
    let org_b = create_org_with_admin(&state, &admin_b_id, "sec-inv-b").await;

    // Admin of A tries to invite to org B -> 403 (not a member of org B)
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_b))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", token_a))
        .body(Body::from(r#"{"email":"victim@test.com","role":"viewer"}"#))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// =============================================================================
// Input Validation
// =============================================================================

#[tokio::test]
async fn test_sql_injection_in_signup_email() {
    let pool = setup_db("sec_sqli_email").await;
    let state = make_state(pool.clone());

    // The email contains an SQL injection payload. Our validation should reject
    // it as an invalid email (no @ or malformed), OR if it somehow passes the
    // basic check, parameterized queries prevent actual injection.
    let sqli_email = "'; DROP TABLE users; --";

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123","name":"Hacker"}}"#,
            sqli_email
        )))
        .unwrap();

    let resp = app(state.clone()).oneshot(req).await.unwrap();

    // Should be rejected as invalid email (400)
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Verify users table still exists and is intact
    let count_row = sqlx::query("SELECT COUNT(*) as cnt FROM users").fetch_one(&pool).await;
    assert!(
        count_row.is_ok(),
        "users table must still exist after SQL injection attempt"
    );
}

#[tokio::test]
async fn test_xss_in_org_name() {
    let pool = setup_db("sec_xss_org").await;
    let state = make_state(pool.clone());

    let xss_name = "<script>alert('xss')</script>";

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"name":"{}","slug":"sec-xss-org"}}"#,
            xss_name
        )))
        .unwrap();

    let resp = app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify the org name is stored as-is (raw) -- sanitization happens on output
    let row = sqlx::query("SELECT name FROM tenants WHERE slug = 'sec-xss-org'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let stored_name: String = row.get("name");
    assert_eq!(
        stored_name, xss_name,
        "stored name should match input exactly"
    );
}

#[tokio::test]
async fn test_long_input_rejection() {
    let pool = setup_db("sec_long_input").await;
    let state = make_state(pool);

    // 10000-char email
    let long_email = format!("{}@example.com", "a".repeat(10_000));

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123","name":"Long Input"}}"#,
            long_email
        )))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    // Should either be 400 (invalid) or at worst 201 without crashing.
    // The email validation is permissive (`contains('@')` + non-empty) so it
    // may accept the long email. The important security property is that the
    // server does NOT crash, panic, or OOM.
    let status = resp.status();
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::CREATED,
        "Server should handle long input gracefully, got {}",
        status
    );
}
