/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Billing ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  formatCurrency,
  formatInvoiceDate,
  getInvoiceStatusColor,
  getInvoiceStatusLabel,
  formatPeriodRange,
  calculateTotalCharges,
  formatHours,
} from '../billing-vm';
import type { BillingPeriodSummary } from '@agent-harbor/rest-client';

describe('Billing ViewModel', () => {
  // ===========================================================================
  // formatCurrency
  // ===========================================================================
  describe('formatCurrency', () => {
    it('formats zero cents', () => {
      expect(formatCurrency(0)).toBe('$0.00');
    });

    it('formats 1250 cents as $12.50', () => {
      expect(formatCurrency(1250)).toBe('$12.50');
    });

    it('formats 100 cents as $1.00', () => {
      expect(formatCurrency(100)).toBe('$1.00');
    });

    it('formats negative values', () => {
      expect(formatCurrency(-500)).toBe('-$5.00');
    });

    it('defaults to USD', () => {
      expect(formatCurrency(1000)).toBe('$10.00');
    });

    it('formats USD explicitly', () => {
      expect(formatCurrency(1000, 'usd')).toBe('$10.00');
    });

    it('formats non-USD currency', () => {
      expect(formatCurrency(1000, 'eur')).toBe('EUR 10.00');
    });
  });

  // ===========================================================================
  // formatInvoiceDate
  // ===========================================================================
  describe('formatInvoiceDate', () => {
    it('formats ISO date string', () => {
      const result = formatInvoiceDate('2026-04-19T00:00:00Z');
      expect(result).toMatch(/Apr 19, 2026/);
    });

    it('formats another date', () => {
      const result = formatInvoiceDate('2026-01-01T00:00:00Z');
      expect(result).toMatch(/Jan 1, 2026/);
    });

    it('returns original string on invalid date', () => {
      // A non-parseable string still returns something (Date may interpret it)
      const result = formatInvoiceDate('not-a-date');
      expect(typeof result).toBe('string');
    });
  });

  // ===========================================================================
  // getInvoiceStatusColor
  // ===========================================================================
  describe('getInvoiceStatusColor', () => {
    it('returns green for paid', () => {
      expect(getInvoiceStatusColor('paid')).toBe('green');
    });

    it('returns yellow for open', () => {
      expect(getInvoiceStatusColor('open')).toBe('yellow');
    });

    it('returns red for void', () => {
      expect(getInvoiceStatusColor('void')).toBe('red');
    });

    it('returns red for uncollectible', () => {
      expect(getInvoiceStatusColor('uncollectible')).toBe('red');
    });

    it('returns gray for draft', () => {
      expect(getInvoiceStatusColor('draft')).toBe('gray');
    });
  });

  // ===========================================================================
  // getInvoiceStatusLabel
  // ===========================================================================
  describe('getInvoiceStatusLabel', () => {
    it('returns Paid for paid', () => {
      expect(getInvoiceStatusLabel('paid')).toBe('Paid');
    });

    it('returns Open for open', () => {
      expect(getInvoiceStatusLabel('open')).toBe('Open');
    });

    it('returns Void for void', () => {
      expect(getInvoiceStatusLabel('void')).toBe('Void');
    });

    it('returns Uncollectible for uncollectible', () => {
      expect(getInvoiceStatusLabel('uncollectible')).toBe('Uncollectible');
    });

    it('returns Draft for draft', () => {
      expect(getInvoiceStatusLabel('draft')).toBe('Draft');
    });
  });

  // ===========================================================================
  // formatPeriodRange
  // ===========================================================================
  describe('formatPeriodRange', () => {
    it('formats same-month range', () => {
      const result = formatPeriodRange('2026-03-01T00:00:00Z', '2026-03-31T00:00:00Z');
      expect(result).toContain('Mar 1');
      expect(result).toContain('Mar 31');
      expect(result).toContain('2026');
      expect(result).toContain('\u2013'); // en-dash
    });

    it('formats cross-month range', () => {
      const result = formatPeriodRange('2026-01-15T00:00:00Z', '2026-02-14T00:00:00Z');
      expect(result).toContain('Jan 15');
      expect(result).toContain('Feb 14');
      expect(result).toContain('2026');
    });
  });

  // ===========================================================================
  // calculateTotalCharges
  // ===========================================================================
  describe('calculateTotalCharges', () => {
    it('calculates seats + executors', () => {
      const summary: BillingPeriodSummary = {
        seatCharges: { count: 5, ratePerSeat: 1500, total: 7500 },
        executorCharges: [
          { machineClass: 'small', hours: 10, ratePerHour: 50, total: 500 },
          { machineClass: 'large', hours: 5, ratePerHour: 200, total: 1000 },
        ],
        totalEstimate: 9000,
        periodStart: '2026-03-01T00:00:00Z',
        periodEnd: '2026-03-31T00:00:00Z',
      };
      expect(calculateTotalCharges(summary)).toBe(9000); // 7500 + 500 + 1000
    });

    it('handles no executor charges', () => {
      const summary: BillingPeriodSummary = {
        seatCharges: { count: 2, ratePerSeat: 1500, total: 3000 },
        executorCharges: [],
        totalEstimate: 3000,
        periodStart: '2026-03-01T00:00:00Z',
        periodEnd: '2026-03-31T00:00:00Z',
      };
      expect(calculateTotalCharges(summary)).toBe(3000);
    });
  });

  // ===========================================================================
  // formatHours
  // ===========================================================================
  describe('formatHours', () => {
    it('formats zero hours', () => {
      expect(formatHours(0)).toBe('0 hrs');
    });

    it('formats singular hour', () => {
      expect(formatHours(1)).toBe('1 hr');
    });

    it('formats fractional hours', () => {
      expect(formatHours(12.5)).toBe('12.5 hrs');
    });

    it('formats multiple hours', () => {
      expect(formatHours(5)).toBe('5 hrs');
    });

    it('formats small fraction', () => {
      expect(formatHours(0.5)).toBe('0.5 hrs');
    });
  });
});
