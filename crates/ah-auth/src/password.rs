// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Argon2id password hashing with OWASP-recommended parameters.

use argon2::{
    Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password hashing failed: {0}")]
    HashError(String),

    #[error("Password verification failed: {0}")]
    VerifyError(String),
}

/// Build an Argon2id hasher with OWASP-recommended parameters.
///
/// Parameters: memory = 19456 KB, iterations = 2, parallelism = 1.
fn argon2_hasher() -> Argon2<'static> {
    let params = Params::new(19456, 2, 1, None).expect("valid Argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hash a password using Argon2id, returning a PHC-format string.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let hasher = argon2_hasher();
    let hash = hasher
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashError(e.to_string()))?;
    Ok(hash.to_string())
}

/// Verify a password against a PHC-format hash string.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| PasswordError::VerifyError(e.to_string()))?;
    let hasher = argon2_hasher();
    match hasher.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerifyError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_argon2id() {
        let password = "correct-horse-battery-staple";
        let hash = hash_password(password).expect("hashing should succeed");

        // PHC format starts with $argon2id$
        assert!(hash.starts_with("$argon2id$"), "should use Argon2id");

        // Correct password should verify
        assert!(
            verify_password(password, &hash).expect("verify should succeed"),
            "correct password must verify"
        );

        // Wrong password should not verify
        assert!(
            !verify_password("wrong-password", &hash).expect("verify should succeed"),
            "wrong password must not verify"
        );
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let hash1 = hash_password("password-one").expect("hash");
        let hash2 = hash_password("password-one").expect("hash");
        // Same password, different salt => different hash strings
        assert_ne!(hash1, hash2, "random salt should produce different hashes");
    }
}
