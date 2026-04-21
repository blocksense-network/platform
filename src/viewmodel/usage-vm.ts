/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Usage Dashboard ViewModel
 *
 * Pure logic for the usage dashboard UI: date range filtering, statistics
 * calculations, formatting helpers, and CSV export generation. No framework
 * dependencies -- consumed by SolidJS route components via simple function calls.
 */

import type { UsageDataPoint, SeatUsageData, ExecutorUsageData } from '@agent-harbor/rest-client';

// =============================================================================
// Types
// =============================================================================

/** Tab selection for the usage dashboard. */
export type UsageTab = 'control-plane' | 'executors';

/** Predefined date range for filtering usage data. */
export type DateRange = '7d' | '30d' | '90d';

// =============================================================================
// Date range filtering
// =============================================================================

/** Maps a DateRange value to the number of days. */
function rangeToDays(range: DateRange): number {
  switch (range) {
    case '7d':
      return 7;
    case '30d':
      return 30;
    case '90d':
      return 90;
  }
}

/**
 * Filters data points to only include those within the given date range.
 *
 * Data points with dates older than `range` days from today are excluded.
 *
 * @param data - Array of usage data points
 * @param range - Date range to filter by
 * @returns Filtered data points within the range
 */
export function filterDataByRange(data: UsageDataPoint[], range: DateRange): UsageDataPoint[] {
  const days = rangeToDays(range);
  const cutoff = new Date();
  cutoff.setDate(cutoff.getDate() - days);
  cutoff.setHours(0, 0, 0, 0);

  return data.filter(dp => new Date(dp.date) >= cutoff);
}

// =============================================================================
// Statistics calculations
// =============================================================================

/**
 * Calculates the arithmetic mean of data point values.
 *
 * @param data - Array of usage data points
 * @returns Average value, or 0 if the array is empty
 */
export function calculateAverage(data: UsageDataPoint[]): number {
  if (data.length === 0) return 0;
  const sum = data.reduce((acc, dp) => acc + dp.value, 0);
  return sum / data.length;
}

/**
 * Returns the maximum value among data points.
 *
 * @param data - Array of usage data points
 * @returns Peak value, or 0 if the array is empty
 */
export function calculatePeak(data: UsageDataPoint[]): number {
  if (data.length === 0) return 0;
  return Math.max(...data.map(dp => dp.value));
}

// =============================================================================
// Formatting helpers
// =============================================================================

/**
 * Formats a usage value with its metric unit.
 *
 * @param value - Numeric value
 * @param metric - Metric name (e.g., "seats", "hours")
 * @returns Formatted string like "5 seats" or "12.5 hrs"
 */
export function formatUsageValue(value: number, metric: string): string {
  if (metric === 'seats') {
    return `${Math.round(value)} ${value === 1 ? 'seat' : 'seats'}`;
  }
  if (metric === 'hours') {
    const rounded = Math.round(value * 10) / 10;
    return `${rounded} ${rounded === 1 ? 'hr' : 'hrs'}`;
  }
  return `${value} ${metric}`;
}

// =============================================================================
// Cost accumulation
// =============================================================================

/**
 * Calculates cumulative cost data points from executor usage data.
 *
 * Takes executor time series data and produces a running total of cost.
 *
 * @param executorData - Executor usage data with per-day values
 * @returns Cumulative cost data points
 */
export function calculateCostAccumulation(executorData: ExecutorUsageData): UsageDataPoint[] {
  let cumulative = 0;
  return executorData.dataPoints.map(dp => {
    cumulative += dp.value;
    return { date: dp.date, value: cumulative };
  });
}

// =============================================================================
// CSV export
// =============================================================================

/**
 * Formats a single CSV row.
 *
 * @param date - Date string
 * @param metric - Metric name
 * @param value - Numeric value
 * @returns Comma-separated row string
 */
export function formatCsvRow(date: string, metric: string, value: number): string {
  return `${date},${metric},${value}`;
}

/**
 * Generates CSV content from seat and executor usage data.
 *
 * Produces a complete CSV string with a header row and data rows for both
 * seat usage and executor usage time series.
 *
 * @param seatData - Seat usage data
 * @param executorData - Executor usage data
 * @returns Complete CSV content string
 */
export function generateCsvContent(
  seatData: SeatUsageData,
  executorData: ExecutorUsageData,
): string {
  const lines: string[] = ['date,metric,value'];

  for (const dp of seatData.dataPoints) {
    lines.push(formatCsvRow(dp.date, 'seats', dp.value));
  }

  for (const dp of executorData.dataPoints) {
    lines.push(formatCsvRow(dp.date, 'executor_hours', dp.value));
  }

  return lines.join('\n');
}
