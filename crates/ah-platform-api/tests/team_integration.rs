// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration tests for team management and invitation endpoints.

use ah_auth::{JwtConfig, RateLimiter};
use ah_platform_api::PlatformApiState;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
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
        secret: "team-integration-test-secret-long-enough-for-hmac-256".to_string(),
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

/// Create a user via the signup endpoint, sign in, create an org, and
/// wire the admin org_member row properly. Returns (access_token, org_id, user_id).
async fn setup_org_with_admin(
    state: &PlatformApiState,
    db_suffix: &str,
    email: &str,
) -> (String, String, String) {
    let router = app(state.clone());

    // Sign up
    let signup_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123","name":"Test User"}}"#,
            email
        )))
        .unwrap();
    let signup_resp = router.clone().oneshot(signup_req).await.unwrap();
    assert_eq!(signup_resp.status(), StatusCode::CREATED);
    let signup_json = body_json(signup_resp.into_body()).await;
    let user_id = signup_json["userId"].as_str().unwrap().to_string();

    // Sign in to get token
    let signin_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123"}}"#,
            email
        )))
        .unwrap();
    let signin_resp = router.clone().oneshot(signin_req).await.unwrap();
    assert_eq!(signin_resp.status(), StatusCode::OK);
    let signin_json = body_json(signin_resp.into_body()).await;
    let access_token = signin_json["accessToken"].as_str().unwrap().to_string();

    // Create org
    let slug = format!("test-org-{}", db_suffix);
    let create_org_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orgs")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"name":"Test Org","slug":"{}","plan":"team"}}"#,
            slug
        )))
        .unwrap();
    let org_resp = router.clone().oneshot(create_org_req).await.unwrap();
    assert_eq!(org_resp.status(), StatusCode::CREATED);
    let org_json = body_json(org_resp.into_body()).await;
    let org_id = org_json["id"].as_str().unwrap().to_string();

    // The create_org handler inserts a "placeholder" user_id in org_members.
    // Update it to the real user_id so our auth checks work.
    sqlx::query(
        "UPDATE org_members SET user_id = $1 WHERE org_id = $2 AND user_id = 'placeholder'",
    )
    .bind(&user_id)
    .bind(&org_id)
    .execute(&state.pool)
    .await
    .unwrap();

    (access_token, org_id, user_id)
}

/// Create a second user, sign up + sign in, return (access_token, user_id).
async fn create_second_user(state: &PlatformApiState, email: &str) -> (String, String) {
    let router = app(state.clone());

    let signup_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123","name":"Second User"}}"#,
            email
        )))
        .unwrap();
    let signup_resp = router.clone().oneshot(signup_req).await.unwrap();
    assert_eq!(signup_resp.status(), StatusCode::CREATED);
    let signup_json = body_json(signup_resp.into_body()).await;
    let user_id = signup_json["userId"].as_str().unwrap().to_string();

    let signin_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"email":"{}","password":"password123"}}"#,
            email
        )))
        .unwrap();
    let signin_resp = router.clone().oneshot(signin_req).await.unwrap();
    assert_eq!(signin_resp.status(), StatusCode::OK);
    let signin_json = body_json(signin_resp.into_body()).await;
    let access_token = signin_json["accessToken"].as_str().unwrap().to_string();

    (access_token, user_id)
}

/// Invite a user and accept the invitation, adding them to the org.
/// Returns the new member's user_id.
async fn invite_and_accept_user(
    state: &PlatformApiState,
    admin_token: &str,
    org_id: &str,
    email: &str,
    role: &str,
) -> String {
    let router = app(state.clone());

    // Send invitation via team endpoint
    let invite_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(format!(
            r#"{{"email":"{}","role":"{}"}}"#,
            email, role
        )))
        .unwrap();
    let invite_resp = router.clone().oneshot(invite_req).await.unwrap();
    assert_eq!(invite_resp.status(), StatusCode::CREATED);
    let invite_json = body_json(invite_resp.into_body()).await;
    let token = invite_json["token"].as_str().unwrap().to_string();

    // Ensure user account exists
    let user_row = sqlx::query("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.pool)
        .await
        .unwrap();

    let user_id = if let Some(row) = user_row {
        let id: String = row.get("id");
        id
    } else {
        // Create user account
        let (_, uid) = create_second_user(state, email).await;
        uid
    };

    // Accept invitation
    let accept_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/auth/invite/{}/accept", token))
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    let accept_resp = router.clone().oneshot(accept_req).await.unwrap();
    assert_eq!(accept_resp.status(), StatusCode::OK);

    user_id
}

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_list_org_members() {
    let pool = setup_db("team_list_members").await;
    let state = make_state(pool);
    let (token, org_id, user_id) =
        setup_org_with_admin(&state, "list", "admin-list@test.com").await;

    let req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    let members = json.as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["userId"], user_id);
    assert_eq!(members[0]["email"], "admin-list@test.com");
    assert_eq!(members[0]["role"], "admin");
}

#[tokio::test]
async fn test_change_member_role() {
    let pool = setup_db("team_change_role").await;
    let state = make_state(pool);
    let (admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "chrole", "admin-chrole@test.com").await;

    // Add a second user as viewer
    let member_id = invite_and_accept_user(
        &state,
        &admin_token,
        &org_id,
        "viewer-chrole@test.com",
        "viewer",
    )
    .await;

    // Change their role to operator
    let req = Request::builder()
        .method("PATCH")
        .uri(&format!("/api/v1/orgs/{}/members/{}", org_id, member_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(r#"{"role":"operator"}"#))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["role"], "operator");
    assert_eq!(json["userId"], member_id);
}

#[tokio::test]
async fn test_change_role_non_admin_forbidden() {
    let pool = setup_db("team_chrole_forbidden").await;
    let state = make_state(pool);
    let (admin_token, org_id, admin_id) =
        setup_org_with_admin(&state, "chrforbid", "admin-forbid@test.com").await;

    // Add second user as viewer
    let _member_id = invite_and_accept_user(
        &state,
        &admin_token,
        &org_id,
        "viewer-forbid@test.com",
        "viewer",
    )
    .await;

    // Get the viewer's token
    let router = app(state.clone());
    let signin_req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/signin")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"email":"viewer-forbid@test.com","password":"password123"}"#,
        ))
        .unwrap();
    let signin_resp = router.oneshot(signin_req).await.unwrap();
    let signin_json = body_json(signin_resp.into_body()).await;
    let viewer_token = signin_json["accessToken"].as_str().unwrap();

    // Viewer tries to change admin's role -> should be 403
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
async fn test_remove_member() {
    let pool = setup_db("team_remove_member").await;
    let state = make_state(pool);
    let (admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "remove", "admin-remove@test.com").await;

    // Add a second user
    let member_id = invite_and_accept_user(
        &state,
        &admin_token,
        &org_id,
        "member-remove@test.com",
        "viewer",
    )
    .await;

    // Remove the member
    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}/members/{}", org_id, member_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify they're gone from members list
    let list_req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let list_resp = app(state).oneshot(list_req).await.unwrap();
    let json = body_json(list_resp.into_body()).await;
    let members = json.as_array().unwrap();
    assert_eq!(members.len(), 1); // only admin remains
}

#[tokio::test]
async fn test_cannot_remove_self() {
    let pool = setup_db("team_remove_self").await;
    let state = make_state(pool);
    let (admin_token, org_id, admin_id) =
        setup_org_with_admin(&state, "rmself", "admin-rmself@test.com").await;

    let req = Request::builder()
        .method("DELETE")
        .uri(&format!("/api/v1/orgs/{}/members/{}", org_id, admin_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_invitation_flow() {
    let pool = setup_db("team_invitation_flow").await;
    let state = make_state(pool);
    let (admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "invflow", "admin-invflow@test.com").await;

    // Create the invited user's account first
    create_second_user(&state, "invited-flow@test.com").await;

    // Send invitation
    let invite_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(
            r#"{"email":"invited-flow@test.com","role":"viewer"}"#,
        ))
        .unwrap();
    let invite_resp = app(state.clone()).oneshot(invite_req).await.unwrap();
    assert_eq!(invite_resp.status(), StatusCode::CREATED);
    let invite_json = body_json(invite_resp.into_body()).await;
    let token = invite_json["token"].as_str().unwrap().to_string();

    // List pending invitations (should show it)
    let list_req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();
    let list_resp = app(state.clone()).oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_json = body_json(list_resp.into_body()).await;
    let pending = list_json.as_array().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["email"], "invited-flow@test.com");

    // Accept invitation
    let accept_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/auth/invite/{}/accept", token))
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    let accept_resp = app(state.clone()).oneshot(accept_req).await.unwrap();
    assert_eq!(accept_resp.status(), StatusCode::OK);

    // List members (should include new member)
    let members_req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/members", org_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();
    let members_resp = app(state.clone()).oneshot(members_req).await.unwrap();
    let members_json = body_json(members_resp.into_body()).await;
    let members = members_json.as_array().unwrap();
    assert_eq!(members.len(), 2);

    // List pending invitations (should be empty now - accepted)
    let list_req2 = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();
    let list_resp2 = app(state).oneshot(list_req2).await.unwrap();
    let list_json2 = body_json(list_resp2.into_body()).await;
    let pending2 = list_json2.as_array().unwrap();
    assert_eq!(pending2.len(), 0);
}

#[tokio::test]
async fn test_seat_limit_enforcement() {
    let pool = setup_db("team_seat_limit").await;
    let state = make_state(pool.clone());

    // Create org with free plan (2 seat limit)
    let (admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "seats", "admin-seats@test.com").await;

    // Update the org to free plan
    sqlx::query("UPDATE tenants SET settings = '{\"plan\":\"free\"}' WHERE id = $1")
        .bind(&org_id)
        .execute(&state.pool)
        .await
        .unwrap();

    // Create user accounts for invitees
    create_second_user(&state, "seat1@test.com").await;
    create_second_user(&state, "seat2@test.com").await;

    // Admin is seat 1. Invite one more user (should succeed - seat 2)
    let invite_req1 = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(r#"{"email":"seat1@test.com","role":"viewer"}"#))
        .unwrap();
    let resp1 = app(state.clone()).oneshot(invite_req1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);

    // Try to invite another (should fail - seat limit reached)
    let invite_req2 = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(r#"{"email":"seat2@test.com","role":"viewer"}"#))
        .unwrap();
    let resp2 = app(state).oneshot(invite_req2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_revoke_invitation() {
    let pool = setup_db("team_revoke_inv").await;
    let state = make_state(pool);
    let (admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "revoke", "admin-revoke@test.com").await;

    // Send invitation
    let invite_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::from(
            r#"{"email":"revokee@test.com","role":"viewer"}"#,
        ))
        .unwrap();
    let invite_resp = app(state.clone()).oneshot(invite_req).await.unwrap();
    assert_eq!(invite_resp.status(), StatusCode::CREATED);
    let invite_json = body_json(invite_resp.into_body()).await;
    let invitation_id = invite_json["id"].as_str().unwrap().to_string();

    // Revoke it
    let revoke_req = Request::builder()
        .method("DELETE")
        .uri(&format!(
            "/api/v1/orgs/{}/team/invitations/{}",
            org_id, invitation_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();
    let revoke_resp = app(state.clone()).oneshot(revoke_req).await.unwrap();
    assert_eq!(revoke_resp.status(), StatusCode::NO_CONTENT);

    // Verify it's gone from pending list
    let list_req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/team/invitations", org_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();
    let list_resp = app(state).oneshot(list_req).await.unwrap();
    let list_json = body_json(list_resp.into_body()).await;
    let pending = list_json.as_array().unwrap();
    assert_eq!(pending.len(), 0);
}

#[tokio::test]
async fn test_accept_expired_invitation() {
    let pool = setup_db("team_expired_inv").await;
    let state = make_state(pool.clone());
    let (_admin_token, org_id, _admin_id) =
        setup_org_with_admin(&state, "expired", "admin-expired@test.com").await;

    // Create user account for the invitee
    create_second_user(&state, "expired-inv@test.com").await;

    // Insert an invitation directly with a past expiry
    let invitation_id = uuid::Uuid::new_v4().to_string();
    let token = ah_auth::generate_invitation_token();
    let past_expiry = "2020-01-01T00:00:00.000Z";

    sqlx::query(
        "INSERT INTO invitations (id, org_id, email, role, token, invited_by, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&invitation_id)
    .bind(&org_id)
    .bind("expired-inv@test.com")
    .bind("viewer")
    .bind(&token)
    .bind(&_admin_id)
    .bind(past_expiry)
    .execute(&state.pool)
    .await
    .unwrap();

    // Try to accept the expired invitation
    let accept_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/auth/invite/{}/accept", token))
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();
    let accept_resp = app(state).oneshot(accept_req).await.unwrap();
    assert_eq!(accept_resp.status(), StatusCode::BAD_REQUEST);
}
