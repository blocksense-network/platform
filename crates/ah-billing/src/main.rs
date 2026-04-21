// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Agent Harbor Billing Service entry point.

use std::sync::Arc;

use ah_billing::db::BillingDb;
use ah_billing::server::{AppState, create_router};
use ah_billing::stripe_client::StripeClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let port = std::env::var("BILLING_PORT").unwrap_or_else(|_| "4100".to_string());
    let db_url =
        std::env::var("BILLING_DATABASE_URL").unwrap_or_else(|_| "sqlite:billing.db".to_string());
    let webhook_secret =
        std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_else(|_| "whsec_test_default".to_string());

    let db = BillingDb::new(&db_url).await?;
    let stripe = StripeClient::from_env().unwrap_or_else(|_| {
        tracing::warn!("STRIPE_SECRET_KEY not set, using dummy client");
        StripeClient::new("https://api.stripe.com", "sk_test_dummy")
    });

    let state = Arc::new(AppState {
        db,
        stripe,
        webhook_secret,
    });

    let app = create_router(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("ah-billing starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
