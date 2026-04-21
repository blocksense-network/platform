// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration tests for the platform authentication endpoints.

use ah_auth::{JwtConfig, RateLimiter};
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

/// Set up an in-memory SQLite database with the minimal schema required by the
/// auth handlers (tenants, users, invitations tables).
async fn setup_db() -> AnyPool {
    ensure_drivers();

    let sqlite_opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);

    let sqlite_pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .connect_with(sqlite_opts)
        .await
        .expect("sqlite connect");

    // Create the minimal schema needed for auth
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
        );
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO tenants (id, name, slug, status) VALUES
            ('00000000-0000-0000-0000-000000000000', 'Default', 'default', 'active');
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
        );
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
        );
        "#,
    )
    .execute(&sqlite_pool)
    .await
    .unwrap();

    // We need an AnyPool but cannot convert SqlitePool directly.
    // Use shared-memory so both the SQLite pool (for schema setup) and the
    // AnyPool (for handlers) point at the same in-memory database.
    drop(sqlite_pool);

    // Re-create with shared memory so we can connect via AnyPool too
    let shared_url = "sqlite:file:auth_test?mode=memory&cache=shared";
    let sqlite_opts2 = SqliteConnectOptions::from_str(shared_url).unwrap().create_if_missing(true);

    let sqlite_pool = SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(2)
        .connect_with(sqlite_opts2)
        .await
        .expect("sqlite shared connect");

    // Create schema on the shared pool
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
        "INSERT OR IGNORE INTO tenants (id, name, slug, status) VALUES ('00000000-0000-0000-0000-000000000000', 'Default', 'default', 'active')",
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

    // Now open AnyPool connected to the same shared-memory DB
    let any_pool = sqlx::any::AnyPoolOptions::new()
        .max_connections(2)
        .connect(shared_url)
        .await
        .expect("any pool connect");

    // Keep sqlite_pool alive (it holds the shared memory database)
    // We leak it intentionally for the test lifetime
    std::mem::forget(sqlite_pool);

    any_pool
}

fn jwt_config() -> JwtConfig {
    JwtConfig {
        secret: "integration-test-secret-long-enough-for-hmac-256".to_string(),
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
    ah_platform_api::auth_router().with_state(state)
}

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_signup_email_password() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());
    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"alice@example.com","password":"strongpassword","name":"Alice"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = body_json(resp.into_body()).await;
    assert!(json["userId"].is_string());
    assert_eq!(json["message"], "Verification email sent");

    // Verify user exists in DB
    let row = sqlx::query("SELECT email, email_verified FROM users WHERE email = $1")
        .bind("alice@example.com")
        .fetch_one(&pool)
        .await
        .unwrap();

    let email: String = sqlx::Row::get(&row, "email");
    assert_eq!(email, "alice@example.com");

    let verified: i32 = sqlx::Row::get(&row, "email_verified");
    assert_eq!(verified, 0);

    // Verify a verification token was generated
    let token_row = sqlx::query(
        "SELECT token FROM invitations WHERE email = $1 AND role = 'email_verification'",
    )
    .bind("alice@example.com")
    .fetch_one(&pool)
    .await
    .unwrap();

    let token: String = sqlx::Row::get(&token_row, "token");
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_signin_email_password() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Create user directly (with verified email)
    let password_hash = ah_auth::hash_password("mypassword123").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, email_verified) VALUES ($1, $2, $3, $4, 1)",
    )
    .bind(&user_id)
    .bind("bob@example.com")
    .bind(&password_hash)
    .bind("Bob")
    .execute(&pool)
    .await
    .unwrap();

    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"bob@example.com","password":"mypassword123"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    assert!(json["accessToken"].is_string());
    assert!(json["refreshToken"].is_string());
    assert_eq!(json["user"]["email"], "bob@example.com");

    // Validate the access token
    let access_token = json["accessToken"].as_str().unwrap();
    let claims = ah_auth::validate_access_token(&jwt_config(), access_token).unwrap();
    assert_eq!(claims.email, "bob@example.com");
}

#[tokio::test]
async fn test_signin_wrong_password() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    let password_hash = ah_auth::hash_password("correctpassword").unwrap();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind("carol@example.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    let app = app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"carol@example.com","password":"wrongpassword"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_signup_duplicate_email() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // First signup
    let app1 = app(state.clone());
    let req1 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"dup@example.com","password":"password123"}"#,
        ))
        .unwrap();
    let resp1 = app1.oneshot(req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Second signup with same email
    let app2 = app(state);
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"dup@example.com","password":"password456"}"#,
        ))
        .unwrap();
    let resp2 = app2.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_password_reset_flow() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Create a user
    let password_hash = ah_auth::hash_password("oldpassword1").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(&user_id)
    .bind("reset@example.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    // Request password reset
    let app1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"reset@example.com"}"#))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Get the token from DB
    let token_row =
        sqlx::query("SELECT token FROM invitations WHERE email = $1 AND role = 'password_reset'")
            .bind("reset@example.com")
            .fetch_one(&pool)
            .await
            .unwrap();
    let reset_token: String = sqlx::Row::get(&token_row, "token");

    // Confirm password reset
    let app2 = app(state.clone());
    let confirm_body = serde_json::json!({
        "token": reset_token,
        "newPassword": "newpassword1"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(confirm_body.to_string()))
        .unwrap();
    let resp = app2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify old password no longer works
    let app3 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"reset@example.com","password":"oldpassword1"}"#,
        ))
        .unwrap();
    let resp = app3.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Verify new password works
    let app4 = app(state);
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"reset@example.com","password":"newpassword1"}"#,
        ))
        .unwrap();
    let resp = app4.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_email_verification() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Sign up
    let app1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"verify@example.com","password":"password123"}"#,
        ))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Get verification token from DB
    let token_row = sqlx::query(
        "SELECT token FROM invitations WHERE email = $1 AND role = 'email_verification'",
    )
    .bind("verify@example.com")
    .fetch_one(&pool)
    .await
    .unwrap();
    let verification_token: String = sqlx::Row::get(&token_row, "token");

    // Verify email
    let app2 = app(state);
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/auth/verify-email/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let resp = app2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Check that email_verified is now 1
    let row = sqlx::query("SELECT email_verified FROM users WHERE email = $1")
        .bind("verify@example.com")
        .fetch_one(&pool)
        .await
        .unwrap();
    let verified: i32 = sqlx::Row::get(&row, "email_verified");
    assert_eq!(verified, 1);
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Create a user
    let password_hash = ah_auth::hash_password("refreshpass1").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(&user_id)
    .bind("refresh@example.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    // Sign in to get tokens
    let app1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"refresh@example.com","password":"refreshpass1"}"#,
        ))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    let refresh_token = json["refreshToken"].as_str().unwrap().to_string();

    // Use refresh token to get new tokens
    let app2 = app(state);
    let refresh_body = serde_json::json!({ "refreshToken": refresh_token });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/refresh")
        .header("content-type", "application/json")
        .body(Body::from(refresh_body.to_string()))
        .unwrap();
    let resp = app2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    assert!(json["accessToken"].is_string());
    assert!(json["refreshToken"].is_string());

    // Validate the new access token
    let new_access = json["accessToken"].as_str().unwrap();
    let claims = ah_auth::validate_access_token(&jwt_config(), new_access).unwrap();
    assert_eq!(claims.email, "refresh@example.com");
}

#[tokio::test]
async fn test_email_verification_token_single_use() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Sign up
    let app1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"singleuse@example.com","password":"password123"}"#,
        ))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Get verification token from DB
    let token_row = sqlx::query(
        "SELECT token FROM invitations WHERE email = $1 AND role = 'email_verification'",
    )
    .bind("singleuse@example.com")
    .fetch_one(&pool)
    .await
    .unwrap();
    let verification_token: String = sqlx::Row::get(&token_row, "token");

    // First use should succeed
    let app2 = app(state.clone());
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/auth/verify-email/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let resp = app2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second use of the same token should fail
    let app3 = app(state);
    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/auth/verify-email/{}", verification_token))
        .body(Body::empty())
        .unwrap();
    let resp = app3.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["message"], "Email already verified");
}

#[tokio::test]
async fn test_password_reset_token_single_use() {
    let pool = setup_db().await;
    let state = make_state(pool.clone());

    // Create a user
    let password_hash = ah_auth::hash_password("original123").unwrap();
    let user_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, email_verified) VALUES ($1, $2, $3, 1)",
    )
    .bind(&user_id)
    .bind("resetreuse@example.com")
    .bind(&password_hash)
    .execute(&pool)
    .await
    .unwrap();

    // Request password reset
    let app1 = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"resetreuse@example.com"}"#))
        .unwrap();
    let resp = app1.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Get the reset token from DB
    let token_row =
        sqlx::query("SELECT token FROM invitations WHERE email = $1 AND role = 'password_reset'")
            .bind("resetreuse@example.com")
            .fetch_one(&pool)
            .await
            .unwrap();
    let reset_token: String = sqlx::Row::get(&token_row, "token");

    // First use should succeed
    let app2 = app(state.clone());
    let confirm_body = serde_json::json!({
        "token": reset_token,
        "newPassword": "newpassword1"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(confirm_body.to_string()))
        .unwrap();
    let resp = app2.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second use of the same token should fail
    let app3 = app(state);
    let confirm_body2 = serde_json::json!({
        "token": reset_token,
        "newPassword": "anotherpass1"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/reset-password/confirm")
        .header("content-type", "application/json")
        .body(Body::from(confirm_body2.to_string()))
        .unwrap();
    let resp = app3.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["message"], "Token already used");
}
