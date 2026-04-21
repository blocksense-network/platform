// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Simple in-memory sliding-window rate limiter keyed by IP address.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Rate limit exceeded — try again later")]
pub struct RateLimitError;

/// In-memory rate limiter that tracks request timestamps per IP.
#[derive(Debug)]
pub struct RateLimiter {
    entries: HashMap<IpAddr, Vec<Instant>>,
}

impl RateLimiter {
    /// Create a new, empty rate limiter.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Check whether the given IP is within the rate limit.
    ///
    /// Records the current instant and returns `Ok(())` if the number of
    /// requests within `window` does not exceed `max_attempts`, or
    /// `Err(RateLimitError)` otherwise.
    pub fn check_rate_limit(
        &mut self,
        ip: IpAddr,
        max_attempts: usize,
        window: Duration,
    ) -> Result<(), RateLimitError> {
        let now = Instant::now();
        let timestamps = self.entries.entry(ip).or_default();

        // Remove expired entries
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= max_attempts {
            return Err(RateLimitError);
        }

        timestamps.push(now);
        Ok(())
    }

    /// Remove all entries whose most recent timestamp is older than `max_age`.
    pub fn cleanup(&mut self, max_age: Duration) {
        let now = Instant::now();
        self.entries.retain(|_, timestamps| {
            timestamps.retain(|t| now.duration_since(*t) < max_age);
            !timestamps.is_empty()
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limiting_auth() {
        let mut limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let window = Duration::from_secs(60);
        let max = 5;

        // First 5 should succeed
        for _ in 0..5 {
            limiter.check_rate_limit(ip, max, window).expect("should be within limit");
        }

        // 6th should fail
        assert!(limiter.check_rate_limit(ip, max, window).is_err());
    }

    #[test]
    fn test_rate_limit_resets_after_window() {
        let mut limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        // Use a tiny window so it expires almost immediately
        let window = Duration::from_millis(50);
        let max = 2;

        // Use up the limit
        limiter.check_rate_limit(ip, max, window).expect("1st ok");
        limiter.check_rate_limit(ip, max, window).expect("2nd ok");
        assert!(
            limiter.check_rate_limit(ip, max, window).is_err(),
            "3rd should fail"
        );

        // Wait for the window to expire
        std::thread::sleep(Duration::from_millis(60));

        // After the window, requests should succeed again
        limiter
            .check_rate_limit(ip, max, window)
            .expect("should succeed after window expires");
    }

    #[test]
    fn test_cleanup_removes_expired() {
        let mut limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        // Add an entry
        limiter.check_rate_limit(ip, 10, Duration::from_secs(60)).unwrap();
        assert!(!limiter.entries.is_empty());

        // Cleanup with zero max_age removes everything
        limiter.cleanup(Duration::ZERO);
        assert!(limiter.entries.is_empty());
    }
}
