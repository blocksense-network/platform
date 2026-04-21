// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Axum HTTP server and route dispatch.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;

use crate::checkout::{self, CheckoutUrls};
use crate::db::BillingDb;
use crate::dunning;
use crate::stripe_client::StripeClient;
use crate::usage::UsageEvent;
use crate::webhooks;

pub struct AppState {
    pub db: BillingDb,
    pub stripe: StripeClient,
    pub webhook_secret: String,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/customers", post(create_customer))
        .route("/api/v1/customers", get(find_customer_by_email))
        .route("/api/v1/customers/{id}", get(get_customer))
        .route("/api/v1/subscriptions", post(create_subscription))
        .route("/api/v1/subscriptions/{id}", get(get_subscription))
        .route("/api/v1/subscriptions/{id}", put(update_subscription))
        .route("/api/v1/subscriptions/{id}", delete(cancel_subscription))
        .route("/api/v1/usage", post(report_usage))
        .route("/api/v1/invoices", get(list_invoices))
        .route("/api/v1/checkout", post(create_checkout))
        .route("/api/v1/portal", post(create_portal))
        .route("/api/v1/webhooks/stripe", post(handle_stripe_webhook))
        .route("/api/v1/dunning/{customer_id}", get(get_dunning))
        .route("/api/v1/admin/evaluate-dunning", post(evaluate_dunning))
        .route("/api/v1/admin/dunning-summary", get(dunning_summary))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// --- Health ---

async fn health() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}

// --- Customers ---

#[derive(Deserialize)]
struct CreateCustomerRequest {
    email: String,
    name: Option<String>,
}

async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCustomerRequest>,
) -> impl IntoResponse {
    match state.db.create_customer(&req.email, req.name.as_deref(), None).await {
        Ok(customer) => (StatusCode::CREATED, Json(json!(customer))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct CustomerQuery {
    email: Option<String>,
}

async fn find_customer_by_email(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CustomerQuery>,
) -> impl IntoResponse {
    let Some(email) = q.email else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing email query parameter"})),
        )
            .into_response();
    };
    match state.db.find_customer_by_email(&email).await {
        Ok(Some(customer)) => Json(json!(customer)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_customer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.get_customer(&id).await {
        Ok(Some(customer)) => Json(json!(customer)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// --- Subscriptions ---

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    customer_id: String,
    plan: String,
    stripe_price_id: Option<String>,
}

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> impl IntoResponse {
    match state
        .db
        .create_subscription(
            &req.customer_id,
            &req.plan,
            None,
            req.stripe_price_id.as_deref(),
        )
        .await
    {
        Ok(sub) => (StatusCode::CREATED, Json(json!(sub))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn get_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.get_subscription(&id).await {
        Ok(Some(sub)) => Json(json!(sub)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct UpdateSubscriptionRequest {
    status: Option<String>,
    plan: Option<String>,
}

async fn update_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSubscriptionRequest>,
) -> impl IntoResponse {
    if let Some(status) = &req.status {
        if let Err(e) = state.db.update_subscription_status(&id, status).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }
    match state.db.get_subscription(&id).await {
        Ok(Some(sub)) => Json(json!(sub)).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn cancel_subscription(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.update_subscription_status(&id, "canceled").await {
        Ok(()) => Json(json!({"status": "canceled"})).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// --- Usage ---

async fn report_usage(
    State(state): State<Arc<AppState>>,
    Json(event): Json<UsageEvent>,
) -> impl IntoResponse {
    match state.db.record_usage(&event).await {
        Ok(()) => (StatusCode::CREATED, Json(json!({"status": "recorded"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// --- Invoices ---

#[derive(Deserialize)]
struct InvoiceQuery {
    #[serde(rename = "customerId")]
    customer_id: Option<String>,
}

async fn list_invoices(
    State(state): State<Arc<AppState>>,
    Query(q): Query<InvoiceQuery>,
) -> impl IntoResponse {
    let Some(customer_id) = q.customer_id else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Missing customerId query parameter"})),
        )
            .into_response();
    };
    match state.db.list_invoices(&customer_id).await {
        Ok(invoices) => Json(json!(invoices)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// --- Checkout & Portal ---

#[derive(Deserialize)]
struct CheckoutRequest {
    customer_stripe_id: String,
    price_id: String,
    success_url: String,
    cancel_url: String,
}

async fn create_checkout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckoutRequest>,
) -> impl IntoResponse {
    let urls = CheckoutUrls {
        success_url: req.success_url,
        cancel_url: req.cancel_url,
    };
    match checkout::create_checkout_session(
        &state.stripe,
        &req.customer_stripe_id,
        &req.price_id,
        &urls,
    )
    .await
    {
        Ok(resp) => Json(json!(resp)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
struct PortalRequest {
    customer_stripe_id: String,
    return_url: String,
}

async fn create_portal(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PortalRequest>,
) -> impl IntoResponse {
    match checkout::create_portal_session(&state.stripe, &req.customer_stripe_id, &req.return_url)
        .await
    {
        Ok(resp) => Json(json!(resp)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// --- Webhooks ---

async fn handle_stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> impl IntoResponse {
    let signature = headers.get("stripe-signature").and_then(|v| v.to_str().ok()).unwrap_or("");

    if !webhooks::verify_webhook_signature(&body, signature, &state.webhook_secret) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid signature"})),
        )
            .into_response();
    }

    let event = match webhooks::parse_webhook_event(&body) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Idempotency check
    match state.db.is_webhook_processed(&event.id).await {
        Ok(true) => {
            return Json(json!({"status": "already_processed"})).into_response();
        }
        Ok(false) => {}
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
        }
    }

    // Mark as processed
    if let Err(e) = state.db.mark_webhook_processed(&event.id, &event.event_type).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response();
    }

    Json(json!({"status": "processed", "event_type": event.event_type})).into_response()
}

// --- Dunning ---

async fn get_dunning(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<String>,
) -> impl IntoResponse {
    match state.db.get_dunning_record(&customer_id).await {
        Ok(Some(record)) => Json(json!({
            "customer_id": record.customer_id,
            "stripe_status": record.stripe_status,
            "state": record.state.as_str(),
            "first_past_due_at": record.first_past_due_at.map(|dt| dt.to_rfc3339()),
            "reminder_count": record.reminder_count,
            "resolved_at": record.resolved_at.map(|dt| dt.to_rfc3339()),
        }))
        .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn evaluate_dunning(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    match state.db.list_all_dunning_records().await {
        Ok(records) => {
            let mut updated = 0;
            for mut record in records {
                let new_state =
                    dunning::evaluate_state(&record.stripe_status, record.first_past_due_at, now);
                if new_state != record.state {
                    record.state = new_state;
                    let _ = state.db.upsert_dunning_record(&record).await;
                    updated += 1;
                }
            }
            Json(json!({"status": "ok", "updated": updated})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn dunning_summary(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.db.list_all_dunning_records().await {
        Ok(records) => {
            let summary: Vec<Value> = records
                .iter()
                .map(|r| {
                    json!({
                        "customer_id": r.customer_id,
                        "state": r.state.as_str(),
                        "stripe_status": r.stripe_status,
                        "reminder_count": r.reminder_count,
                    })
                })
                .collect();
            Json(json!(summary)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
