// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! 5-state dunning state machine.
//! Follows patterns from metacraft-billing/src/dunning.nim.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DunningState {
    Active,
    PastDue,
    Restricted,
    Suspended,
    Canceled,
}

impl DunningState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Restricted => "restricted",
            Self::Suspended => "suspended",
            Self::Canceled => "canceled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "past_due" => Self::PastDue,
            "restricted" => Self::Restricted,
            "suspended" => Self::Suspended,
            "canceled" => Self::Canceled,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DunningRecord {
    pub id: String,
    pub customer_id: String,
    pub stripe_subscription_id: Option<String>,
    pub stripe_status: String,
    pub state: DunningState,
    pub first_past_due_at: Option<DateTime<Utc>>,
    pub last_reminder_at: Option<DateTime<Utc>>,
    pub reminder_count: i32,
    pub resolved_at: Option<DateTime<Utc>>,
}

fn days_since(first_past_due_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> i64 {
    match first_past_due_at {
        Some(dt) => (now - dt).num_days(),
        None => 0,
    }
}

/// Derives dunning state from Stripe status + elapsed time:
/// - active -> Active
/// - past_due, <7 days -> PastDue (grace period)
/// - past_due, 7-21 days -> Restricted
/// - past_due, 21-90 days -> Suspended
/// - past_due, >90 days -> Canceled
/// - canceled -> Canceled
pub fn evaluate_state(
    stripe_status: &str,
    first_past_due_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> DunningState {
    match stripe_status {
        "active" => DunningState::Active,
        "past_due" => {
            let days = days_since(first_past_due_at, now);
            if days >= 90 {
                DunningState::Canceled
            } else if days >= 21 {
                DunningState::Suspended
            } else if days >= 7 {
                DunningState::Restricted
            } else {
                DunningState::PastDue
            }
        }
        "canceled" => DunningState::Canceled,
        _ => DunningState::Active,
    }
}

/// Called when we receive a subscription.updated webhook from Stripe.
/// Mutates the record in-place based on the new status.
pub fn on_stripe_status_changed(record: &mut DunningRecord, new_status: &str, now: DateTime<Utc>) {
    record.stripe_status = new_status.to_string();
    match new_status {
        "past_due" => {
            if record.first_past_due_at.is_none() {
                record.first_past_due_at = Some(now);
            }
            record.state = evaluate_state("past_due", record.first_past_due_at, now);
        }
        "active" => {
            record.first_past_due_at = None;
            record.last_reminder_at = None;
            record.reminder_count = 0;
            record.resolved_at = Some(now);
            record.state = DunningState::Active;
        }
        "canceled" => {
            record.state = DunningState::Canceled;
        }
        _ => {
            record.state = evaluate_state(new_status, record.first_past_due_at, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_dunning_state_machine() {
        let now = Utc::now();

        // active status -> Active
        assert_eq!(evaluate_state("active", None, now), DunningState::Active);

        // past_due, 0 days -> PastDue
        assert_eq!(
            evaluate_state("past_due", Some(now), now),
            DunningState::PastDue
        );

        // past_due, 10 days -> Restricted
        let ten_days_ago = now - Duration::days(10);
        assert_eq!(
            evaluate_state("past_due", Some(ten_days_ago), now),
            DunningState::Restricted
        );

        // past_due, 30 days -> Suspended
        let thirty_days_ago = now - Duration::days(30);
        assert_eq!(
            evaluate_state("past_due", Some(thirty_days_ago), now),
            DunningState::Suspended
        );

        // past_due, 100 days -> Canceled
        let hundred_days_ago = now - Duration::days(100);
        assert_eq!(
            evaluate_state("past_due", Some(hundred_days_ago), now),
            DunningState::Canceled
        );

        // canceled status -> Canceled
        assert_eq!(
            evaluate_state("canceled", None, now),
            DunningState::Canceled
        );

        // Recovery: past_due -> payment succeeds (status back to active) -> Active
        let mut record = DunningRecord {
            id: "d1".into(),
            customer_id: "c1".into(),
            stripe_subscription_id: None,
            stripe_status: "past_due".into(),
            state: DunningState::PastDue,
            first_past_due_at: Some(now - Duration::days(5)),
            last_reminder_at: None,
            reminder_count: 1,
            resolved_at: None,
        };
        on_stripe_status_changed(&mut record, "active", now);
        assert_eq!(record.state, DunningState::Active);
        assert!(record.first_past_due_at.is_none());
        assert_eq!(record.reminder_count, 0);
        assert!(record.resolved_at.is_some());
    }
}
