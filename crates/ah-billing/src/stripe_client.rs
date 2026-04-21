// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Stripe API client using HTTP Basic auth and form-encoded params.
//! Follows the patterns from metacraft-billing/src/stripe_client.nim.

use base64::Engine;
use reqwest::{Client, Method};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StripeError {
    #[error("Stripe API error (HTTP {status}): {message}")]
    Api {
        status: u16,
        message: String,
        code: Option<String>,
    },
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Configuration error: {0}")]
    Config(String),
}

pub struct StripeClient {
    base_url: String,
    secret_key: String,
    http: Client,
}

impl StripeClient {
    pub fn new(base_url: &str, secret_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            secret_key: secret_key.to_string(),
            http: Client::new(),
        }
    }

    /// Create from env vars: STRIPE_API_URL (default https://api.stripe.com), STRIPE_SECRET_KEY (required).
    pub fn from_env() -> Result<Self, StripeError> {
        let base_url = std::env::var("STRIPE_API_URL")
            .unwrap_or_else(|_| "https://api.stripe.com".to_string());
        let secret_key = std::env::var("STRIPE_SECRET_KEY")
            .map_err(|_| StripeError::Config("STRIPE_SECRET_KEY not set".into()))?;
        Ok(Self::new(&base_url, &secret_key))
    }

    /// HTTP Basic auth header: username = secret key, password = empty.
    pub fn auth_header(&self) -> String {
        let encoded =
            base64::engine::general_purpose::STANDARD.encode(format!("{}:", self.secret_key));
        format!("Basic {}", encoded)
    }

    /// Make an authenticated request. Stripe uses HTTP Basic with key as username, empty password.
    /// Content-Type: application/x-www-form-urlencoded
    pub async fn request(
        &self,
        method: Method,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<Value, StripeError> {
        let url = format!("{}{}", self.base_url, path);

        let mut builder = self
            .http
            .request(method.clone(), &url)
            .header("Authorization", self.auth_header());

        match method {
            Method::GET => {
                if !params.is_empty() {
                    builder = builder.query(params);
                }
            }
            _ => {
                builder = builder.header("Content-Type", "application/x-www-form-urlencoded").body(
                    params
                        .iter()
                        .map(|(k, v)| {
                            format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
                        })
                        .collect::<Vec<_>>()
                        .join("&"),
                );
            }
        }

        let response = builder.send().await?;
        let status = response.status().as_u16();
        let body_text = response.text().await?;

        let body_json: Value = serde_json::from_str(&body_text)
            .unwrap_or_else(|_| serde_json::json!({"raw": body_text}));

        if status >= 400 {
            let code = body_json
                .get("error")
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
            return Err(StripeError::Api {
                status,
                message: body_text,
                code,
            });
        }

        Ok(body_json)
    }

    pub async fn create_customer(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<Value, StripeError> {
        let mut params: Vec<(&str, &str)> = vec![("email", email)];
        if let Some(n) = name {
            params.push(("name", n));
        }
        self.request(Method::POST, "/v1/customers", &params).await
    }

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
    ) -> Result<Value, StripeError> {
        self.request(
            Method::POST,
            "/v1/subscriptions",
            &[("customer", customer_id), ("items[0][price]", price_id)],
        )
        .await
    }

    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<Value, StripeError> {
        self.request(
            Method::DELETE,
            &format!("/v1/subscriptions/{}", subscription_id),
            &[],
        )
        .await
    }

    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        success_url: &str,
        cancel_url: &str,
    ) -> Result<Value, StripeError> {
        self.request(
            Method::POST,
            "/v1/checkout/sessions",
            &[
                ("customer", customer_id),
                ("line_items[0][price]", price_id),
                ("line_items[0][quantity]", "1"),
                ("mode", "subscription"),
                ("success_url", success_url),
                ("cancel_url", cancel_url),
            ],
        )
        .await
    }

    pub async fn create_portal_session(
        &self,
        customer_id: &str,
        return_url: &str,
    ) -> Result<Value, StripeError> {
        self.request(
            Method::POST,
            "/v1/billing_portal/sessions",
            &[("customer", customer_id), ("return_url", return_url)],
        )
        .await
    }

    pub async fn create_usage_record(
        &self,
        subscription_item_id: &str,
        quantity: u64,
        timestamp: i64,
    ) -> Result<Value, StripeError> {
        let qty_str = quantity.to_string();
        let ts_str = timestamp.to_string();
        self.request(
            Method::POST,
            &format!(
                "/v1/subscription_items/{}/usage_records",
                subscription_item_id
            ),
            &[("quantity", &qty_str), ("timestamp", &ts_str)],
        )
        .await
    }

    pub async fn list_invoices(&self, customer_id: &str) -> Result<Value, StripeError> {
        self.request(Method::GET, "/v1/invoices", &[("customer", customer_id)]).await
    }
}

// We use urlencoding for form body encoding. Add a minimal inline implementation
// to avoid adding another dependency.
mod urlencoding {
    pub fn encode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for byte in input.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push('%');
                    result.push_str(&format!("{:02X}", byte));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_client_auth_header() {
        let client = StripeClient::new("https://api.stripe.com", "sk_test_123");
        let header = client.auth_header();
        // "sk_test_123:" base64 encoded
        let expected_encoded = base64::engine::general_purpose::STANDARD.encode("sk_test_123:");
        assert_eq!(header, format!("Basic {}", expected_encoded));
    }

    #[test]
    fn test_stripe_client_form_encoding() {
        let encoded = urlencoding::encode("hello world&foo=bar");
        assert_eq!(encoded, "hello%20world%26foo%3Dbar");
    }
}
