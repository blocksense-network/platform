// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Webhook handling and signature verification.
//! Follows patterns from metacraft-billing/src/webhooks.nim.

use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub id: String,
    pub event_type: String,
    pub data: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum WebhookError {
    #[error("Invalid webhook payload: {0}")]
    InvalidPayload(String),
}

/// Parse a Stripe webhook event from the JSON body.
pub fn parse_webhook_event(body: &str) -> Result<WebhookEvent, WebhookError> {
    let j: Value =
        serde_json::from_str(body).map_err(|e| WebhookError::InvalidPayload(e.to_string()))?;

    let id = j
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebhookError::InvalidPayload("missing id".into()))?
        .to_string();

    let event_type = j
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| WebhookError::InvalidPayload("missing type".into()))?
        .to_string();

    let data = j.get("data").cloned().unwrap_or(Value::Null);

    Ok(WebhookEvent {
        id,
        event_type,
        data,
    })
}

/// Verify a Stripe webhook signature.
/// If secret starts with "whsec_test_", always return true (localstripe compat).
/// Otherwise, compute HMAC-SHA256 and compare against the v1 signature.
pub fn verify_webhook_signature(body: &str, signature: &str, secret: &str) -> bool {
    // Test mode: always pass
    if secret.starts_with("whsec_test_") {
        return true;
    }

    if signature.is_empty() {
        return false;
    }

    // Parse signature header: t=<timestamp>,v1=<sig>
    let mut timestamp = None;
    let mut v1_sig = None;

    for part in signature.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            match key {
                "t" if !value.is_empty() => timestamp = Some(value.to_string()),
                "v1" if !value.is_empty() => v1_sig = Some(value.to_string()),
                _ => {}
            }
        }
    }

    let (Some(ts), Some(sig)) = (timestamp, v1_sig) else {
        return false;
    };

    // Compute expected signature: HMAC-SHA256(secret, "timestamp.body")
    let signed_payload = format!("{}.{}", ts, body);
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(signed_payload.as_bytes());
    let result = mac.finalize();
    let expected = hex::encode(result.into_bytes());

    expected == sig
}

// Inline hex encoding to avoid extra dependency
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signature_verification() {
        // whsec_test_ prefix should always pass
        assert!(verify_webhook_signature("body", "any", "whsec_test_secret"));

        // Empty signature should fail
        assert!(!verify_webhook_signature("body", "", "whsec_live_secret"));

        // Valid format with t= and v1= should pass structure check
        // (We compute the real HMAC here for a proper round-trip test)
        let secret = "test_secret";
        let body = r#"{"id":"evt_1"}"#;
        let timestamp = "1234567890";

        // Compute the correct HMAC
        let signed_payload = format!("{}.{}", timestamp, body);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed_payload.as_bytes());
        let sig_bytes = mac.finalize().into_bytes();
        let sig_hex = hex::encode(sig_bytes);

        let signature = format!("t={},v1={}", timestamp, sig_hex);
        assert!(verify_webhook_signature(body, &signature, secret));

        // Wrong body should fail
        assert!(!verify_webhook_signature("wrong body", &signature, secret));
    }

    #[test]
    fn test_parse_webhook_event() {
        let body = r#"{
            "id": "evt_test_123",
            "type": "customer.subscription.updated",
            "data": {"object": {"id": "sub_123", "status": "active"}}
        }"#;

        let event = parse_webhook_event(body).unwrap();
        assert_eq!(event.id, "evt_test_123");
        assert_eq!(event.event_type, "customer.subscription.updated");
        assert_eq!(event.data["object"]["status"].as_str().unwrap(), "active");
    }

    #[tokio::test]
    async fn test_webhook_idempotency() {
        let db = crate::db::BillingDb::in_memory().await.unwrap();

        assert!(!db.is_webhook_processed("evt_1").await.unwrap());
        db.mark_webhook_processed("evt_1", "test.event").await.unwrap();
        assert!(db.is_webhook_processed("evt_1").await.unwrap());
    }
}
