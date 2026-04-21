/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Admin Revenue ViewModel
 *
 * Pure logic for the admin revenue dashboard UI: revenue split calculation,
 * growth rate computation, and formatting. No framework dependencies --
 * consumed by SolidJS route components via simple function calls.
 */

// =============================================================================
// Revenue split
// =============================================================================

/** Revenue split percentages. */
export interface RevenueSplit {
  seatPercent: number;
  executorPercent: number;
}

/**
 * Calculates the percentage split between seat and executor revenue.
 *
 * @param seatRevenue - Seat revenue in cents
 * @param executorRevenue - Executor revenue in cents
 * @returns Split percentages (each 0-100), or both 0 if no revenue
 */
export function formatRevenueSplit(seatRevenue: number, executorRevenue: number): RevenueSplit {
  const total = seatRevenue + executorRevenue;
  if (total === 0) {
    return { seatPercent: 0, executorPercent: 0 };
  }
  const seatPercent = Math.round((seatRevenue / total) * 100);
  const executorPercent = 100 - seatPercent;
  return { seatPercent, executorPercent };
}

// =============================================================================
// Growth rate
// =============================================================================

/**
 * Calculates the growth rate as a percentage.
 *
 * @param current - Current period value
 * @param previous - Previous period value
 * @returns Growth rate as a percentage (e.g., 12.5 for 12.5% growth)
 */
export function calculateGrowthRate(current: number, previous: number): number {
  if (previous === 0) {
    return current > 0 ? 100 : 0;
  }
  return ((current - previous) / previous) * 100;
}

/**
 * Formats a growth rate for display with sign prefix.
 *
 * @example formatGrowthRate(12.5) => "+12.5%"
 * @example formatGrowthRate(-3.2) => "-3.2%"
 * @example formatGrowthRate(0) => "0.0%"
 */
export function formatGrowthRate(rate: number): string {
  const formatted = Math.abs(rate).toFixed(1);
  if (rate > 0) {
    return `+${formatted}%`;
  }
  if (rate < 0) {
    return `-${formatted}%`;
  }
  return `${formatted}%`;
}
