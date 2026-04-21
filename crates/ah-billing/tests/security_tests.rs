// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Security tests for billing webhook signature verification and idempotency.

use ah_billing::webhooks::{parse_webhook_event, verify_webhook_signature};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute a valid Stripe-format signature for the given body and secret.
fn compute_stripe_signature(body: &str, secret: &str, timestamp: &str) -> String {
    let signed_payload = format!("{}.{}", timestamp, body);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(signed_payload.as_bytes());
    let result = mac.finalize();
    let sig_hex: String = result.into_bytes().iter().map(|b| format!("{:02x}", b)).collect();
    format!("t={},v1={}", timestamp, sig_hex)
}

// =============================================================================
// Webhook Signature Security
// =============================================================================

#[test]
fn test_webhook_without_signature() {
    let body = r#"{"id":"evt_1","type":"test.event","data":{}}"#;
    let secret = "whsec_live_real_secret";

    // Empty signature string should be rejected
    assert!(
        !verify_webhook_signature(body, "", secret),
        "Empty signature must be rejected"
    );
}

#[test]
fn test_webhook_invalid_signature() {
    let body = r#"{"id":"evt_1","type":"test.event","data":{}}"#;
    let secret = "whsec_live_real_secret";

    // Malformed signature (wrong HMAC value)
    let bad_sig =
        "t=1234567890,v1=0000000000000000000000000000000000000000000000000000000000000000";
    assert!(
        !verify_webhook_signature(body, bad_sig, secret),
        "Invalid HMAC signature must be rejected"
    );

    // Missing v1 component
    assert!(
        !verify_webhook_signature(body, "t=1234567890", secret),
        "Signature without v1 component must be rejected"
    );

    // Missing timestamp component
    assert!(
        !verify_webhook_signature(body, "v1=abcdef", secret),
        "Signature without timestamp must be rejected"
    );

    // Completely garbage string
    assert!(
        !verify_webhook_signature(body, "not-a-signature-at-all", secret),
        "Garbage signature must be rejected"
    );
}

#[test]
fn test_webhook_test_mode_bypass() {
    let body = r#"{"id":"evt_test","type":"test.event","data":{}}"#;
    let test_secret = "whsec_test_local_development";

    // Test-mode secret (whsec_test_ prefix) should always pass,
    // regardless of the signature content
    assert!(
        verify_webhook_signature(body, "any-signature-value", test_secret),
        "Test-mode secret must bypass signature verification"
    );

    assert!(
        verify_webhook_signature(body, "", test_secret),
        "Test-mode secret must bypass even with empty signature"
    );
}

#[test]
fn test_webhook_valid_signature_accepted() {
    let body = r#"{"id":"evt_valid","type":"checkout.session.completed","data":{}}"#;
    let secret = "real_webhook_secret";
    let timestamp = "1700000000";

    let signature = compute_stripe_signature(body, secret, timestamp);
    assert!(
        verify_webhook_signature(body, &signature, secret),
        "Valid signature must be accepted"
    );
}

#[test]
fn test_webhook_body_tampered_after_signing() {
    let original_body = r#"{"id":"evt_1","type":"test.event","data":{}}"#;
    let tampered_body = r#"{"id":"evt_1","type":"test.event","data":{"hacked":true}}"#;
    let secret = "real_webhook_secret";
    let timestamp = "1700000000";

    // Sign the original body
    let signature = compute_stripe_signature(original_body, secret, timestamp);

    // Verify with tampered body must fail
    assert!(
        !verify_webhook_signature(tampered_body, &signature, secret),
        "Tampered body must not pass signature verification"
    );
}

#[tokio::test]
async fn test_webhook_duplicate_event_rejected() {
    let db = ah_billing::db::BillingDb::in_memory().await.unwrap();

    let event_id = "evt_duplicate_test";

    // First time: not processed
    assert!(
        !db.is_webhook_processed(event_id).await.unwrap(),
        "Event should not be marked as processed initially"
    );

    // Mark as processed
    db.mark_webhook_processed(event_id, "test.event").await.unwrap();

    // Second time: should be marked as processed (idempotency check)
    assert!(
        db.is_webhook_processed(event_id).await.unwrap(),
        "Event should be marked as processed after first handling"
    );

    // Marking again should not error (INSERT OR IGNORE)
    db.mark_webhook_processed(event_id, "test.event").await.unwrap();
    assert!(
        db.is_webhook_processed(event_id).await.unwrap(),
        "Event should still be processed after duplicate mark"
    );
}

#[test]
fn test_webhook_parse_missing_fields() {
    // Missing "id"
    let body_no_id = r#"{"type":"test.event","data":{}}"#;
    assert!(
        parse_webhook_event(body_no_id).is_err(),
        "Webhook without id field must fail parsing"
    );

    // Missing "type"
    let body_no_type = r#"{"id":"evt_1","data":{}}"#;
    assert!(
        parse_webhook_event(body_no_type).is_err(),
        "Webhook without type field must fail parsing"
    );

    // Invalid JSON
    let body_invalid = "not json at all";
    assert!(
        parse_webhook_event(body_invalid).is_err(),
        "Invalid JSON must fail parsing"
    );
}
