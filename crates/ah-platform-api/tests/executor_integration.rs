// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration tests for managed executor endpoints.

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

/// Set up an in-memory SQLite database with the full platform schema
/// including managed_executors and executor_usage_records tables.
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
            provisioned_at TEXT,
            terminated_at TEXT,
            hourly_rate_cents INTEGER,
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
        CREATE TABLE IF NOT EXISTS executor_usage_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            org_id TEXT NOT NULL,
            executor_id TEXT NOT NULL,
            period_start TEXT NOT NULL,
            period_end TEXT,
            duration_seconds INTEGER,
            reported_to_stripe INTEGER NOT NULL DEFAULT 0,
            stripe_usage_record_id TEXT,
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
        secret: "executor-integration-test-secret-long-enough-for-hmac-256".to_string(),
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

/// Create a user, sign in, create an org with the given plan, and wire up
/// the admin org_member row. Returns (access_token, org_id, user_id).
async fn setup_org_with_plan(
    state: &PlatformApiState,
    db_suffix: &str,
    email: &str,
    plan: &str,
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
            r#"{{"name":"Test Org","slug":"{}","plan":"{}"}}"#,
            slug, plan
        )))
        .unwrap();
    let org_resp = router.clone().oneshot(create_org_req).await.unwrap();
    assert_eq!(org_resp.status(), StatusCode::CREATED);
    let org_json = body_json(org_resp.into_body()).await;
    let org_id = org_json["id"].as_str().unwrap().to_string();

    // Fix placeholder user_id in org_members
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

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_list_machine_classes() {
    let pool = setup_db("exec_catalog").await;
    let state = make_state(pool);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/catalog/machine-classes")
        .body(Body::empty())
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_json(resp.into_body()).await;
    let classes = json.as_array().unwrap();
    assert_eq!(classes.len(), 4);

    // Verify standard class
    let standard = classes.iter().find(|c| c["id"] == "standard").unwrap();
    assert_eq!(standard["vcpus"], 4);
    assert_eq!(standard["ramGb"], 16);
    assert_eq!(standard["storageGb"], 100);
    assert_eq!(standard["storageType"], "SSD");
    assert_eq!(standard["hourlyRateCents"], 50);

    // Verify GPU class
    let gpu = classes.iter().find(|c| c["id"] == "gpu").unwrap();
    assert_eq!(gpu["vcpus"], 8);
    assert_eq!(gpu["ramGb"], 32);
    assert_eq!(gpu["hourlyRateCents"], 300);

    // Verify high-memory class
    let highmem = classes.iter().find(|c| c["id"] == "high-memory").unwrap();
    assert_eq!(highmem["ramGb"], 64);
    assert_eq!(highmem["storageGb"], 500);

    // Verify performance class
    let perf = classes.iter().find(|c| c["id"] == "performance").unwrap();
    assert_eq!(perf["hourlyRateCents"], 100);
}

#[tokio::test]
async fn test_provision_executor() {
    let pool = setup_db("exec_provision").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "provision", "admin-prov@test.com", "team").await;

    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"standard","os":"linux","region":"us-east","name":"my-executor"}"#,
        ))
        .unwrap();

    let resp = app(state.clone()).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = body_json(resp.into_body()).await;
    assert_eq!(json["machineClass"], "standard");
    assert_eq!(json["os"], "linux");
    assert_eq!(json["region"], "us-east");
    assert_eq!(json["name"], "my-executor");
    assert_eq!(json["status"], "provisioning");
    assert_eq!(json["hourlyRateCents"], 50);
    assert_eq!(json["orgId"], org_id);

    // Verify DB record
    let executor_id = json["id"].as_str().unwrap();
    let row = sqlx::query("SELECT status, machine_class FROM managed_executors WHERE id = $1")
        .bind(executor_id)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    let status: String = row.get("status");
    let mc: String = row.get("machine_class");
    assert_eq!(status, "provisioning");
    assert_eq!(mc, "standard");
}

#[tokio::test]
async fn test_provision_invalid_machine_class() {
    let pool = setup_db("exec_inv_class").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "invclass", "admin-invclass@test.com", "team").await;

    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"nonexistent","os":"linux","region":"us-east"}"#,
        ))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let json = body_json(resp.into_body()).await;
    assert!(json["message"].as_str().unwrap().contains("Unknown machine class"));
}

#[tokio::test]
async fn test_provision_invalid_os() {
    let pool = setup_db("exec_inv_os").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "invos", "admin-invos@test.com", "team").await;

    // "high-memory" only supports linux, not windows
    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"high-memory","os":"windows","region":"us-east"}"#,
        ))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let json = body_json(resp.into_body()).await;
    assert!(json["message"].as_str().unwrap().contains("not available for machine class"));
}

#[tokio::test]
async fn test_executor_lifecycle() {
    let pool = setup_db("exec_lifecycle").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "lifecycle", "admin-lifecycle@test.com", "team").await;

    // Provision
    let provision_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"performance","os":"linux","region":"eu-west"}"#,
        ))
        .unwrap();
    let provision_resp = app(state.clone()).oneshot(provision_req).await.unwrap();
    assert_eq!(provision_resp.status(), StatusCode::CREATED);
    let provision_json = body_json(provision_resp.into_body()).await;
    let executor_id = provision_json["id"].as_str().unwrap().to_string();
    assert_eq!(provision_json["status"], "provisioning");

    // List — should show provisioning
    let list_req = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .body(Body::empty())
        .unwrap();
    let list_resp = app(state.clone()).oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
    let list_json = body_json(list_resp.into_body()).await;
    let executors = list_json.as_array().unwrap();
    assert_eq!(executors.len(), 1);
    assert_eq!(executors[0]["status"], "provisioning");

    // Simulate manual update to running (as if infra provisioning completed)
    sqlx::query("UPDATE managed_executors SET status = 'running' WHERE id = $1")
        .bind(&executor_id)
        .execute(&state.pool)
        .await
        .unwrap();

    // Stop
    let stop_req = Request::builder()
        .method("POST")
        .uri(&format!(
            "/api/v1/orgs/{}/executors/{}/stop",
            org_id, executor_id
        ))
        .body(Body::empty())
        .unwrap();
    let stop_resp = app(state.clone()).oneshot(stop_req).await.unwrap();
    assert_eq!(stop_resp.status(), StatusCode::OK);
    let stop_json = body_json(stop_resp.into_body()).await;
    assert_eq!(stop_json["status"], "stopped");

    // Start
    let start_req = Request::builder()
        .method("POST")
        .uri(&format!(
            "/api/v1/orgs/{}/executors/{}/start",
            org_id, executor_id
        ))
        .body(Body::empty())
        .unwrap();
    let start_resp = app(state.clone()).oneshot(start_req).await.unwrap();
    assert_eq!(start_resp.status(), StatusCode::OK);
    let start_json = body_json(start_resp.into_body()).await;
    assert_eq!(start_json["status"], "running");

    // Terminate
    let term_req = Request::builder()
        .method("DELETE")
        .uri(&format!(
            "/api/v1/orgs/{}/executors/{}",
            org_id, executor_id
        ))
        .body(Body::empty())
        .unwrap();
    let term_resp = app(state.clone()).oneshot(term_req).await.unwrap();
    assert_eq!(term_resp.status(), StatusCode::OK);
    let term_json = body_json(term_resp.into_body()).await;
    assert_eq!(term_json["status"], "terminated");
    assert!(term_json["terminatedAt"].as_str().is_some());

    // List — should still show the terminated executor
    let list_req2 = Request::builder()
        .method("GET")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .body(Body::empty())
        .unwrap();
    let list_resp2 = app(state).oneshot(list_req2).await.unwrap();
    let list_json2 = body_json(list_resp2.into_body()).await;
    let executors2 = list_json2.as_array().unwrap();
    assert_eq!(executors2.len(), 1);
    assert_eq!(executors2[0]["status"], "terminated");
}

#[tokio::test]
async fn test_free_plan_cannot_provision() {
    let pool = setup_db("exec_free_plan").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "freeplan", "admin-free@test.com", "free").await;

    let req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"standard","os":"linux","region":"us-east"}"#,
        ))
        .unwrap();

    let resp = app(state).oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let json = body_json(resp.into_body()).await;
    assert!(json["message"].as_str().unwrap().contains("free plan"));
}

#[tokio::test]
async fn test_record_executor_usage() {
    let pool = setup_db("exec_usage_record").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "usagerec", "admin-usage@test.com", "team").await;

    // Provision an executor
    let provision_req = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"standard","os":"linux","region":"us-east"}"#,
        ))
        .unwrap();
    let provision_resp = app(state.clone()).oneshot(provision_req).await.unwrap();
    assert_eq!(provision_resp.status(), StatusCode::CREATED);
    let provision_json = body_json(provision_resp.into_body()).await;
    let executor_id = provision_json["id"].as_str().unwrap().to_string();

    // Record usage
    ah_platform_api::executor_metering::record_executor_usage(
        &state.pool,
        &executor_id,
        &org_id,
        3600,
    )
    .await
    .unwrap();

    // Verify the record exists in DB
    let row =
        sqlx::query("SELECT duration_seconds FROM executor_usage_records WHERE executor_id = $1")
            .bind(&executor_id)
            .fetch_one(&state.pool)
            .await
            .unwrap();
    let duration: i64 = row.get("duration_seconds");
    assert_eq!(duration, 3600);
}

#[tokio::test]
async fn test_get_org_executor_usage_summary() {
    let pool = setup_db("exec_usage_summary").await;
    let state = make_state(pool);
    let (_token, org_id, _user_id) =
        setup_org_with_plan(&state, "usagesum", "admin-usagesum@test.com", "team").await;

    // Provision two executors of different classes
    let provision1 = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"standard","os":"linux","region":"us-east"}"#,
        ))
        .unwrap();
    let resp1 = app(state.clone()).oneshot(provision1).await.unwrap();
    assert_eq!(resp1.status(), StatusCode::CREATED);
    let json1 = body_json(resp1.into_body()).await;
    let exec1_id = json1["id"].as_str().unwrap().to_string();

    let provision2 = Request::builder()
        .method("POST")
        .uri(&format!("/api/v1/orgs/{}/executors", org_id))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"machineClass":"performance","os":"linux","region":"eu-west"}"#,
        ))
        .unwrap();
    let resp2 = app(state.clone()).oneshot(provision2).await.unwrap();
    assert_eq!(resp2.status(), StatusCode::CREATED);
    let json2 = body_json(resp2.into_body()).await;
    let exec2_id = json2["id"].as_str().unwrap().to_string();

    // Record usage for both
    ah_platform_api::executor_metering::record_executor_usage(
        &state.pool,
        &exec1_id,
        &org_id,
        3600, // 1 hour
    )
    .await
    .unwrap();

    ah_platform_api::executor_metering::record_executor_usage(
        &state.pool,
        &exec1_id,
        &org_id,
        1800, // 30 min
    )
    .await
    .unwrap();

    ah_platform_api::executor_metering::record_executor_usage(
        &state.pool,
        &exec2_id,
        &org_id,
        7200, // 2 hours
    )
    .await
    .unwrap();

    // Get summary from the beginning of time
    let summaries = ah_platform_api::executor_metering::get_org_executor_usage(
        &state.pool,
        &org_id,
        "2000-01-01T00:00:00.000Z",
    )
    .await
    .unwrap();

    assert_eq!(summaries.len(), 2);

    let standard = summaries.iter().find(|s| s.machine_class == "standard").unwrap();
    assert_eq!(standard.total_seconds, 5400); // 3600 + 1800
    // Cost: (5400 / 3600) * 50 = 75 cents
    assert_eq!(standard.total_cost_cents, 75);

    let perf = summaries.iter().find(|s| s.machine_class == "performance").unwrap();
    assert_eq!(perf.total_seconds, 7200);
    // Cost: (7200 / 3600) * 100 = 200 cents
    assert_eq!(perf.total_cost_cents, 200);
}
