// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Cryptographically secure random token generation.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;

/// Generate a cryptographically random URL-safe base64 token of `len` bytes.
pub fn generate_token(len: usize) -> String {
    let mut buf = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(&buf)
}

/// Generate a 32-byte verification token for email verification.
pub fn generate_verification_token() -> String {
    generate_token(32)
}

/// Generate a 32-byte token for password reset.
pub fn generate_reset_token() -> String {
    generate_token(32)
}

/// Generate a 32-byte invitation token.
pub fn generate_invitation_token() -> String {
    generate_token(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let t1 = generate_token(32);
        let t2 = generate_token(32);

        // 32 bytes -> 43 chars in URL-safe base64 (no padding)
        assert_eq!(t1.len(), 43, "32 bytes encodes to 43 base64 chars");
        assert_eq!(t2.len(), 43);

        // Tokens must be unique
        assert_ne!(t1, t2, "tokens should be unique");

        // Must be URL-safe (no +, /, =)
        for ch in t1.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_',
                "token char '{}' is not URL-safe",
                ch
            );
        }
    }

    #[test]
    fn test_convenience_token_lengths() {
        let v = generate_verification_token();
        let r = generate_reset_token();
        let i = generate_invitation_token();
        // All should be 43 chars (32 bytes in base64)
        assert_eq!(v.len(), 43);
        assert_eq!(r.len(), 43);
        assert_eq!(i.len(), 43);
    }
}
