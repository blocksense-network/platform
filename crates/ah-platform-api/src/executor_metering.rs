// Copyright 2025 Schelling Point Labs Inc
// SPDX-License-Identifier: AGPL-3.0-only

//! Usage metering for managed executors.

use serde::Serialize;
use sqlx::{AnyPool, Row};

/// Summary of executor usage for a single machine class within a billing period.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorUsageSummary {
    pub machine_class: String,
    pub total_seconds: i64,
    pub total_cost_cents: i64,
}

/// Record a usage event for a running executor.
pub async fn record_executor_usage(
    pool: &AnyPool,
    executor_id: &str,
    org_id: &str,
    duration_seconds: i64,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    sqlx::query(
        "INSERT INTO executor_usage_records (org_id, executor_id, period_start, duration_seconds, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(org_id)
    .bind(executor_id)
    .bind(&now)
    .bind(duration_seconds)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get total usage aggregated by machine class for an org since `period_start`.
pub async fn get_org_executor_usage(
    pool: &AnyPool,
    org_id: &str,
    period_start: &str,
) -> Result<Vec<ExecutorUsageSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT me.machine_class, SUM(eur.duration_seconds) as total_seconds, me.hourly_rate_cents
         FROM executor_usage_records eur
         JOIN managed_executors me ON me.id = eur.executor_id
         WHERE eur.org_id = $1 AND eur.created_at >= $2
         GROUP BY me.machine_class, me.hourly_rate_cents",
    )
    .bind(org_id)
    .bind(period_start)
    .fetch_all(pool)
    .await?;

    let summaries = rows
        .iter()
        .map(|row| {
            let machine_class: String = row.get("machine_class");
            let total_seconds: i64 = row.get("total_seconds");
            let hourly_rate_cents: i64 = row.get("hourly_rate_cents");
            // Cost = (total_seconds / 3600) * hourly_rate_cents
            let total_cost_cents = (total_seconds * hourly_rate_cents) / 3600;
            ExecutorUsageSummary {
                machine_class,
                total_seconds,
                total_cost_cents,
            }
        })
        .collect();

    Ok(summaries)
}
