// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Usage metering: record, aggregate, and report usage events.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub customer_id: String,
    pub product_id: String,
    pub metric: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAggregate {
    pub customer_id: String,
    pub product_id: String,
    pub metric: String,
    pub total_value: f64,
    pub event_ids: Vec<i64>,
}

pub async fn record_usage(pool: &SqlitePool, event: &UsageEvent) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO usage_events (customer_id, product_id, metric, value) VALUES (?, ?, ?, ?)",
    )
    .bind(&event.customer_id)
    .bind(&event.product_id)
    .bind(&event.metric)
    .bind(event.value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_unreported_usage(
    pool: &SqlitePool,
    customer_id: &str,
) -> Result<Vec<UsageAggregate>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, f64)>(
        "SELECT id, customer_id, product_id, metric, value FROM usage_events \
         WHERE customer_id = ? AND reported_to_stripe = 0 \
         ORDER BY product_id, metric",
    )
    .bind(customer_id)
    .fetch_all(pool)
    .await?;

    // Aggregate by (product_id, metric)
    let mut aggregates: std::collections::HashMap<(String, String), UsageAggregate> =
        std::collections::HashMap::new();

    for (id, cust_id, product_id, metric, value) in rows {
        let key = (product_id.clone(), metric.clone());
        let agg = aggregates.entry(key).or_insert_with(|| UsageAggregate {
            customer_id: cust_id,
            product_id,
            metric,
            total_value: 0.0,
            event_ids: Vec::new(),
        });
        agg.total_value += value;
        agg.event_ids.push(id);
    }

    Ok(aggregates.into_values().collect())
}

pub async fn mark_reported(pool: &SqlitePool, ids: &[i64]) -> Result<(), sqlx::Error> {
    for id in ids {
        sqlx::query("UPDATE usage_events SET reported_to_stripe = 1 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::BillingDb;

    #[tokio::test]
    async fn test_usage_aggregation() {
        let db = BillingDb::in_memory().await.unwrap();
        let pool = db.pool();

        // Record multiple events
        let e1 = UsageEvent {
            customer_id: "c1".into(),
            product_id: "p1".into(),
            metric: "api_calls".into(),
            value: 10.0,
        };
        let e2 = UsageEvent {
            customer_id: "c1".into(),
            product_id: "p1".into(),
            metric: "api_calls".into(),
            value: 5.0,
        };
        let e3 = UsageEvent {
            customer_id: "c1".into(),
            product_id: "p1".into(),
            metric: "storage_mb".into(),
            value: 100.0,
        };

        record_usage(pool, &e1).await.unwrap();
        record_usage(pool, &e2).await.unwrap();
        record_usage(pool, &e3).await.unwrap();

        // Get unreported
        let aggregates = get_unreported_usage(pool, "c1").await.unwrap();
        assert_eq!(aggregates.len(), 2);

        let api_agg = aggregates.iter().find(|a| a.metric == "api_calls").unwrap();
        assert!((api_agg.total_value - 15.0).abs() < f64::EPSILON);
        assert_eq!(api_agg.event_ids.len(), 2);

        // Mark reported
        let all_ids: Vec<i64> =
            aggregates.iter().flat_map(|a| a.event_ids.iter().copied()).collect();
        mark_reported(pool, &all_ids).await.unwrap();

        // Verify empty after marking
        let after = get_unreported_usage(pool, "c1").await.unwrap();
        assert!(after.is_empty());
    }
}
