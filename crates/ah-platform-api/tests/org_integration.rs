// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration tests for the platform organization endpoints.

use ah_auth::{AccessTokenClaims, JwtConfig, RateLimiter, create_access_token};
use ah_platform_api::PlatformApiState;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::{
    AnyPool,
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

/// Set up an in-memory SQLite database with the schema required by the
/// org handlers (tenants, users, invitations, org_members tables).
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
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
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
        secret: "org-integration-test-secret-long-enough-for-hmac-256".to_string(),
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

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_create_org() {
    let pool = setup_db("org_create_test").await;
    let state = make_state(pool.clone());
    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"name":"Acme Corp","slug":"acme-corp","plan":"team"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = body_json(resp.into_body()).await;
    assert!(json["id"].is_string());
    assert_eq!(json["name"], "Acme Corp");
    assert_eq!(json["slug"], "acme-corp");
    assert_eq!(json["plan"], "team");
    assert!(json["createdAt"].is_string());

    // Verify tenant row exists in DB
    let tenant_row = sqlx::query("SELECT name, slug, settings FROM tenants WHERE slug = $1")
        .bind("acme-corp")
        .fetch_one(&pool)
        .await
        .unwrap();

    let name: String = sqlx::Row::get(&tenant_row, "name");
    assert_eq!(name, "Acme Corp");

    let slug: String = sqlx::Row::get(&tenant_row, "slug");
    assert_eq!(slug, "acme-corp");

    // Verify org_members row with admin role
    let org_id = json["id"].as_str().unwrap();
    let member_row = sqlx::query("SELECT role FROM org_members WHERE org_id = $1")
        .bind(org_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let role: String = sqlx::Row::get(&member_row, "role");
    assert_eq!(role, "admin");
}

#[tokio::test]
async fn test_create_org_duplicate_slug() {
    let pool = setup_db("org_dup_slug_test").await;
    let state = make_state(pool.clone());

    // First org
    let app1 = app(state.clone());
    let req1 = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Acme Corp","slug":"acme-corp"}"#))
        .unwrap();
    let resp1 = app1.oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second org with same slug
    let app2 = app(state);
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Another Acme","slug":"acme-corp"}"#))
        .unwrap();
    let resp2 = app2.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_create_org_invalid_slug() {
    let pool = setup_db("org_invalid_slug_test").await;
    let state = make_state(pool.clone());
    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Test","slug":"A"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_org_empty_name() {
    let pool = setup_db("org_empty_name_test").await;
    let state = make_state(pool.clone());
    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"","slug":"test-org"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// M13: Organization Settings Tests
// ============================================================================

/// Helper: create an org and set up a user + org_member row, returning
/// (org_id, bearer_token).
async fn create_org_with_auth(pool: &AnyPool, slug: &str) -> (String, String) {
    let user_id = uuid::Uuid::new_v4().to_string();
    let org_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    // Insert user
    sqlx::query("INSERT INTO users (id, email, name) VALUES ($1, $2, $3)")
        .bind(&user_id)
        .bind(&format!("{}@test.com", slug))
        .bind("Test User")
        .execute(pool)
        .await
        .unwrap();

    // Insert tenant
    sqlx::query(
        "INSERT INTO tenants (id, name, slug, status, settings, created_at, updated_at)
         VALUES ($1, $2, $3, 'active', '{\"plan\":\"team\"}', $4, $5)",
    )
    .bind(&org_id)
    .bind(&format!("{} Corp", slug))
    .bind(slug)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();

    // Insert org_member as admin
    let member_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO org_members (id, org_id, user_id, role, created_at)
         VALUES ($1, $2, $3, 'admin', $4)",
    )
    .bind(&member_id)
    .bind(&org_id)
    .bind(&user_id)
    .bind(&now)
    .execute(pool)
    .await
    .unwrap();

    // Create JWT
    let config = jwt_config();
    let exp = chrono::Utc::now().timestamp() + 900;
    let claims = AccessTokenClaims {
        sub: user_id,
        email: format!("{}@test.com", slug),
        org_id: Some(org_id.clone()),
        role: Some("admin".to_string()),
        exp,
        iat: chrono::Utc::now().timestamp(),
    };
    let token = create_access_token(&config, &claims).unwrap();

    (org_id, format!("Bearer {}", token))
}

#[tokio::test]
async fn test_get_org_settings() {
    let pool = setup_db("org_get_settings_test").await;
    let state = make_state(pool.clone());
    let (org_id, bearer) = create_org_with_auth(&pool, "settings-get").await;

    let app = app(state);
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}", org_id))
        .header("Authorization", &bearer)
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["id"], org_id);
    assert_eq!(json["slug"], "settings-get");
    assert_eq!(json["plan"], "team");
    assert!(json["name"].is_string());
    assert!(json["createdAt"].is_string());
}

#[tokio::test]
async fn test_update_org_name() {
    let pool = setup_db("org_update_name_test").await;
    let state = make_state(pool.clone());
    let (org_id, bearer) = create_org_with_auth(&pool, "update-name").await;

    let app = app(state);
    let req = Request::builder()
        .method("PATCH")
        .uri(&format!("/api/v1/orgs/{}", org_id))
        .header("content-type", "application/json")
        .header("Authorization", &bearer)
        .body(Body::from(r#"{"name":"New Name"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["name"], "New Name");
    assert_eq!(json["slug"], "update-name");
}

#[tokio::test]
async fn test_delete_org() {
    let pool = setup_db("org_delete_test").await;
    let state = make_state(pool.clone());
    let (org_id, bearer) = create_org_with_auth(&pool, "delete-me").await;

    let app = app(state);
    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}", org_id))
        .header("content-type", "application/json")
        .header("Authorization", &bearer)
        .body(Body::from(r#"{"confirmSlug":"delete-me"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify tenant status is 'deleted'
    let row = sqlx::query("SELECT status FROM tenants WHERE id = $1")
        .bind(&org_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let status: String = sqlx::Row::get(&row, "status");
    assert_eq!(status, "deleted");
}

#[tokio::test]
async fn test_delete_org_wrong_slug() {
    let pool = setup_db("org_delete_wrong_slug_test").await;
    let state = make_state(pool.clone());
    let (org_id, bearer) = create_org_with_auth(&pool, "keep-me").await;

    let app = app(state);
    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}", org_id))
        .header("content-type", "application/json")
        .header("Authorization", &bearer)
        .body(Body::from(r#"{"confirmSlug":"wrong-slug"}"#))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
