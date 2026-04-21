// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Agent Harbor Billing
//!
//! Standalone billing microservice with Stripe integration, dunning state machine,
//! usage metering, and webhook handling.

pub mod checkout;
pub mod db;
pub mod dunning;
pub mod server;
pub mod stripe_client;
pub mod usage;
pub mod webhooks;
