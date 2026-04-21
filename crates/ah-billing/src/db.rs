// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Database layer using SQLite via sqlx.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use uuid::Uuid;

use crate::dunning::{DunningRecord, DunningState};
use crate::usage::UsageEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub customer_id: String,
    pub stripe_subscription_id: Option<String>,
    pub stripe_price_id: Option<String>,
    pub plan: String,
    pub status: String,
    pub current_period_start: Option<String>,
    pub current_period_end: Option<String>,
    pub cancel_at_period_end: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceCacheEntry {
    pub id: String,
    pub customer_id: String,
    pub stripe_invoice_id: Option<String>,
    pub number: Option<String>,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub currency: String,
    pub status: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub pdf_url: Option<String>,
    pub created_at: String,
}

const MIGRATION_SQL: &str = include_str!("../migrations/001_initial.up.sql");

pub struct BillingDb {
    pool: SqlitePool,
}

impl BillingDb {
    /// Create a new database connection pool, run migrations.
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let options: SqliteConnectOptions =
            url.parse::<SqliteConnectOptions>()?.create_if_missing(true);
        let pool = SqlitePoolOptions::new().max_connections(5).connect_with(options).await?;

        sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
        // Run migrations
        sqlx::raw_sql(MIGRATION_SQL).execute(&pool).await?;

        Ok(Self { pool })
    }

    /// In-memory database for tests.
    pub async fn in_memory() -> Result<Self, sqlx::Error> {
        Self::new("sqlite::memory:").await
    }

    /// Access the underlying pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // --- Customer CRUD ---

    pub async fn create_customer(
        &self,
        email: &str,
        name: Option<&str>,
        stripe_customer_id: Option<&str>,
    ) -> Result<Customer, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO customers (id, email, name, stripe_customer_id, created_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(email)
        .bind(name)
        .bind(stripe_customer_id)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Customer {
            id,
            email: email.to_string(),
            name: name.map(String::from),
            stripe_customer_id: stripe_customer_id.map(String::from),
            created_at: now,
        })
    }

    pub async fn get_customer(&self, id: &str) -> Result<Option<Customer>, sqlx::Error> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String)>(
            "SELECT id, email, name, stripe_customer_id, created_at FROM customers WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            row.map(|(id, email, name, stripe_id, created_at)| Customer {
                id,
                email,
                name,
                stripe_customer_id: stripe_id,
                created_at,
            }),
        )
    }

    pub async fn find_customer_by_email(
        &self,
        email: &str,
    ) -> Result<Option<Customer>, sqlx::Error> {
        let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String)>(
            "SELECT id, email, name, stripe_customer_id, created_at FROM customers WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(
            row.map(|(id, email, name, stripe_id, created_at)| Customer {
                id,
                email,
                name,
                stripe_customer_id: stripe_id,
                created_at,
            }),
        )
    }

    // --- Subscription CRUD ---

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        plan: &str,
        stripe_subscription_id: Option<&str>,
        stripe_price_id: Option<&str>,
    ) -> Result<Subscription, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO subscriptions (id, customer_id, stripe_subscription_id, stripe_price_id, \
             plan, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(customer_id)
        .bind(stripe_subscription_id)
        .bind(stripe_price_id)
        .bind(plan)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(Subscription {
            id,
            customer_id: customer_id.to_string(),
            stripe_subscription_id: stripe_subscription_id.map(String::from),
            stripe_price_id: stripe_price_id.map(String::from),
            plan: plan.to_string(),
            status: "active".to_string(),
            current_period_start: None,
            current_period_end: None,
            cancel_at_period_end: false,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn get_subscription(&self, id: &str) -> Result<Option<Subscription>, sqlx::Error> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                String,
                String,
                Option<String>,
                Option<String>,
                bool,
                String,
                String,
            ),
        >(
            "SELECT id, customer_id, stripe_subscription_id, stripe_price_id, plan, status, \
             current_period_start, current_period_end, cancel_at_period_end, created_at, updated_at \
             FROM subscriptions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                id,
                customer_id,
                stripe_sub_id,
                stripe_price_id,
                plan,
                status,
                period_start,
                period_end,
                cancel,
                created_at,
                updated_at,
            )| Subscription {
                id,
                customer_id,
                stripe_subscription_id: stripe_sub_id,
                stripe_price_id,
                plan,
                status,
                current_period_start: period_start,
                current_period_end: period_end,
                cancel_at_period_end: cancel,
                created_at,
                updated_at,
            },
        ))
    }

    pub async fn update_subscription_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE subscriptions SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Dunning ---

    pub async fn get_dunning_record(
        &self,
        customer_id: &str,
    ) -> Result<Option<DunningRecord>, sqlx::Error> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
                Option<String>,
                i32,
                Option<String>,
            ),
        >(
            "SELECT id, customer_id, stripe_subscription_id, stripe_status, state, \
             first_past_due_at, last_reminder_at, reminder_count, resolved_at \
             FROM dunning_records WHERE customer_id = ?",
        )
        .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(
            |(
                id,
                customer_id,
                stripe_sub_id,
                stripe_status,
                state_str,
                first_past_due,
                last_reminder,
                reminder_count,
                resolved,
            )| {
                DunningRecord {
                    id,
                    customer_id,
                    stripe_subscription_id: stripe_sub_id,
                    stripe_status,
                    state: DunningState::from_str(&state_str),
                    first_past_due_at: first_past_due
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    last_reminder_at: last_reminder
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    reminder_count,
                    resolved_at: resolved
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                }
            },
        ))
    }

    pub async fn upsert_dunning_record(&self, record: &DunningRecord) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO dunning_records (id, customer_id, stripe_subscription_id, stripe_status, \
             state, first_past_due_at, last_reminder_at, reminder_count, resolved_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
             stripe_status = excluded.stripe_status, \
             state = excluded.state, \
             first_past_due_at = excluded.first_past_due_at, \
             last_reminder_at = excluded.last_reminder_at, \
             reminder_count = excluded.reminder_count, \
             resolved_at = excluded.resolved_at",
        )
        .bind(&record.id)
        .bind(&record.customer_id)
        .bind(&record.stripe_subscription_id)
        .bind(&record.stripe_status)
        .bind(record.state.as_str())
        .bind(record.first_past_due_at.map(|dt| dt.to_rfc3339()))
        .bind(record.last_reminder_at.map(|dt| dt.to_rfc3339()))
        .bind(record.reminder_count)
        .bind(record.resolved_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // --- Webhook idempotency ---

    pub async fn is_webhook_processed(&self, event_id: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM webhook_events WHERE id = ?")
            .bind(event_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 > 0)
    }

    pub async fn mark_webhook_processed(
        &self,
        event_id: &str,
        event_type: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO webhook_events (id, event_type) VALUES (?, ?)")
            .bind(event_id)
            .bind(event_type)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Usage ---

    pub async fn record_usage(&self, event: &UsageEvent) -> Result<(), sqlx::Error> {
        crate::usage::record_usage(&self.pool, event).await
    }

    // --- Invoices cache ---

    pub async fn cache_invoice(
        &self,
        customer_id: &str,
        stripe_invoice_id: Option<&str>,
        number: Option<&str>,
        amount_due: i64,
        amount_paid: i64,
        currency: &str,
        status: &str,
        period_start: Option<&str>,
        period_end: Option<&str>,
        pdf_url: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO invoices_cache (id, customer_id, stripe_invoice_id, number, \
             amount_due, amount_paid, currency, status, period_start, period_end, pdf_url, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(customer_id)
        .bind(stripe_invoice_id)
        .bind(number)
        .bind(amount_due)
        .bind(amount_paid)
        .bind(currency)
        .bind(status)
        .bind(period_start)
        .bind(period_end)
        .bind(pdf_url)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_invoices(
        &self,
        customer_id: &str,
    ) -> Result<Vec<InvoiceCacheEntry>, sqlx::Error> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
                String,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                String,
            ),
        >(
            "SELECT id, customer_id, stripe_invoice_id, number, amount_due, amount_paid, \
             currency, status, period_start, period_end, pdf_url, created_at \
             FROM invoices_cache WHERE customer_id = ? ORDER BY created_at DESC",
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    customer_id,
                    stripe_invoice_id,
                    number,
                    amount_due,
                    amount_paid,
                    currency,
                    status,
                    period_start,
                    period_end,
                    pdf_url,
                    created_at,
                )| InvoiceCacheEntry {
                    id,
                    customer_id,
                    stripe_invoice_id,
                    number,
                    amount_due,
                    amount_paid,
                    currency,
                    status,
                    period_start,
                    period_end,
                    pdf_url,
                    created_at,
                },
            )
            .collect())
    }

    // --- All dunning records (for admin) ---

    pub async fn list_all_dunning_records(&self) -> Result<Vec<DunningRecord>, sqlx::Error> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<String>,
                String,
                String,
                Option<String>,
                Option<String>,
                i32,
                Option<String>,
            ),
        >(
            "SELECT id, customer_id, stripe_subscription_id, stripe_status, state, \
             first_past_due_at, last_reminder_at, reminder_count, resolved_at \
             FROM dunning_records",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    customer_id,
                    stripe_sub_id,
                    stripe_status,
                    state_str,
                    first_past_due,
                    last_reminder,
                    reminder_count,
                    resolved,
                )| DunningRecord {
                    id,
                    customer_id,
                    stripe_subscription_id: stripe_sub_id,
                    stripe_status,
                    state: DunningState::from_str(&state_str),
                    first_past_due_at: first_past_due
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    last_reminder_at: last_reminder
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    reminder_count,
                    resolved_at: resolved
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                },
            )
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_customer_crud() {
        let db = BillingDb::in_memory().await.unwrap();

        // Create
        let customer =
            db.create_customer("test@example.com", Some("Test User"), None).await.unwrap();
        assert_eq!(customer.email, "test@example.com");
        assert_eq!(customer.name.as_deref(), Some("Test User"));

        // Get by ID
        let fetched = db.get_customer(&customer.id).await.unwrap().unwrap();
        assert_eq!(fetched.email, "test@example.com");

        // Find by email
        let found = db.find_customer_by_email("test@example.com").await.unwrap().unwrap();
        assert_eq!(found.id, customer.id);

        // Not found
        let missing = db.find_customer_by_email("nobody@example.com").await.unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_subscription_crud() {
        let db = BillingDb::in_memory().await.unwrap();

        let customer = db.create_customer("sub@example.com", None, None).await.unwrap();

        // Create subscription
        let sub = db
            .create_subscription(&customer.id, "pro", Some("sub_stripe_1"), Some("price_1"))
            .await
            .unwrap();
        assert_eq!(sub.plan, "pro");
        assert_eq!(sub.status, "active");

        // Get
        let fetched = db.get_subscription(&sub.id).await.unwrap().unwrap();
        assert_eq!(fetched.plan, "pro");

        // Update status
        db.update_subscription_status(&sub.id, "past_due").await.unwrap();
        let updated = db.get_subscription(&sub.id).await.unwrap().unwrap();
        assert_eq!(updated.status, "past_due");
    }

    #[tokio::test]
    async fn test_invoice_cache() {
        let db = BillingDb::in_memory().await.unwrap();

        let customer = db.create_customer("inv@example.com", None, None).await.unwrap();

        // Cache an invoice
        db.cache_invoice(
            &customer.id,
            Some("inv_stripe_1"),
            Some("INV-001"),
            2900,
            2900,
            "usd",
            "paid",
            Some("2025-01-01"),
            Some("2025-02-01"),
            Some("https://stripe.com/invoice.pdf"),
        )
        .await
        .unwrap();

        // List invoices
        let invoices = db.list_invoices(&customer.id).await.unwrap();
        assert_eq!(invoices.len(), 1);
        assert_eq!(invoices[0].amount_due, 2900);
        assert_eq!(invoices[0].status, "paid");
        assert_eq!(
            invoices[0].stripe_invoice_id.as_deref(),
            Some("inv_stripe_1")
        );
    }
}
