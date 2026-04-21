// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Checkout and portal session creation.

use serde::{Deserialize, Serialize};

use crate::stripe_client::{StripeClient, StripeError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutUrls {
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub session_id: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalResponse {
    pub url: String,
}

pub async fn create_checkout_session(
    stripe: &StripeClient,
    customer_stripe_id: &str,
    price_id: &str,
    urls: &CheckoutUrls,
) -> Result<CheckoutResponse, StripeError> {
    let result = stripe
        .create_checkout_session(
            customer_stripe_id,
            price_id,
            &urls.success_url,
            &urls.cancel_url,
        )
        .await?;

    Ok(CheckoutResponse {
        session_id: result["id"].as_str().unwrap_or("").to_string(),
        url: result["url"].as_str().unwrap_or("").to_string(),
    })
}

pub async fn create_portal_session(
    stripe: &StripeClient,
    customer_stripe_id: &str,
    return_url: &str,
) -> Result<PortalResponse, StripeError> {
    let result = stripe.create_portal_session(customer_stripe_id, return_url).await?;

    Ok(PortalResponse {
        url: result["url"].as_str().unwrap_or("").to_string(),
    })
}
