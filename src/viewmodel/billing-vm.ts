/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Billing ViewModel
 *
 * Pure logic for the billing & invoices UI: currency formatting, invoice status
 * display, period formatting, and charge calculations. No framework dependencies
 * -- consumed by SolidJS route components via simple function calls.
 */

import type { BillingPeriodSummary } from '@agent-harbor/rest-client';

// =============================================================================
// Types
// =============================================================================

/** Invoice status values. */
export type InvoiceStatus = 'draft' | 'open' | 'paid' | 'void' | 'uncollectible';

// =============================================================================
// Currency formatting
// =============================================================================

/**
 * Formats an amount in cents as a currency string.
 * @example formatCurrency(1250) => "$12.50"
 * @example formatCurrency(0) => "$0.00"
 */
export function formatCurrency(cents: number, currency: string = 'usd'): string {
  const amount = cents / 100;
  const symbol = currency.toLowerCase() === 'usd' ? '$' : currency.toUpperCase() + ' ';
  // Use toFixed(2) for consistent formatting
  const absAmount = Math.abs(amount);
  const formatted = `${symbol}${absAmount.toFixed(2)}`;
  return cents < 0 ? `-${formatted}` : formatted;
}

// =============================================================================
// Date formatting
// =============================================================================

/**
 * Formats an ISO date string for invoice display.
 * @example formatInvoiceDate("2026-04-19T00:00:00Z") => "Apr 19, 2026"
 */
export function formatInvoiceDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateStr;
  }
}

/**
 * Formats a billing period date range for display.
 * @example formatPeriodRange("2026-03-01T00:00:00Z", "2026-03-31T00:00:00Z") => "Mar 1 – Mar 31, 2026"
 */
export function formatPeriodRange(start: string, end: string): string {
  try {
    const startDate = new Date(start);
    const endDate = new Date(end);

    const startMonth = startDate.toLocaleDateString('en-US', { month: 'short' });
    const endMonth = endDate.toLocaleDateString('en-US', { month: 'short' });
    const startDay = startDate.getUTCDate();
    const endDay = endDate.getUTCDate();
    const endYear = endDate.getUTCFullYear();

    return `${startMonth} ${startDay} \u2013 ${endMonth} ${endDay}, ${endYear}`;
  } catch {
    return `${start} – ${end}`;
  }
}

// =============================================================================
// Invoice status display
// =============================================================================

/**
 * Returns a Tailwind color class name for an invoice status badge.
 */
export function getInvoiceStatusColor(status: InvoiceStatus): string {
  switch (status) {
    case 'paid':
      return 'green';
    case 'open':
      return 'yellow';
    case 'void':
    case 'uncollectible':
      return 'red';
    case 'draft':
    default:
      return 'gray';
  }
}

/**
 * Returns a human-readable label for an invoice status.
 */
export function getInvoiceStatusLabel(status: InvoiceStatus): string {
  switch (status) {
    case 'paid':
      return 'Paid';
    case 'open':
      return 'Open';
    case 'void':
      return 'Void';
    case 'uncollectible':
      return 'Uncollectible';
    case 'draft':
      return 'Draft';
    default:
      return status;
  }
}

// =============================================================================
// Charge calculations
// =============================================================================

/**
 * Calculates total charges from a billing period summary.
 */
export function calculateTotalCharges(summary: BillingPeriodSummary): number {
  const seatTotal = summary.seatCharges.total;
  const executorTotal = summary.executorCharges.reduce((sum, charge) => sum + charge.total, 0);
  return seatTotal + executorTotal;
}

/**
 * Formats hours for display.
 * @example formatHours(1) => "1 hr"
 * @example formatHours(12.5) => "12.5 hrs"
 * @example formatHours(0) => "0 hrs"
 */
export function formatHours(hours: number): string {
  if (hours === 1) return '1 hr';
  return `${hours} hrs`;
}
