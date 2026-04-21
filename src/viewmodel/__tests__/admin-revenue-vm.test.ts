/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Admin Revenue ViewModel Tests (M12)
 */

import { describe, it, expect } from 'vitest';
import { formatRevenueSplit, calculateGrowthRate, formatGrowthRate } from '../admin-revenue-vm';

describe('Admin Revenue ViewModel', () => {
  // ===========================================================================
  // formatRevenueSplit
  // ===========================================================================
  describe('formatRevenueSplit', () => {
    it('returns 0/0 when both are zero', () => {
      const split = formatRevenueSplit(0, 0);
      expect(split.seatPercent).toBe(0);
      expect(split.executorPercent).toBe(0);
    });

    it('calculates even split', () => {
      const split = formatRevenueSplit(5000, 5000);
      expect(split.seatPercent).toBe(50);
      expect(split.executorPercent).toBe(50);
    });

    it('calculates 75/25 split', () => {
      const split = formatRevenueSplit(7500, 2500);
      expect(split.seatPercent).toBe(75);
      expect(split.executorPercent).toBe(25);
    });

    it('handles 100% seat revenue', () => {
      const split = formatRevenueSplit(10000, 0);
      expect(split.seatPercent).toBe(100);
      expect(split.executorPercent).toBe(0);
    });

    it('handles 100% executor revenue', () => {
      const split = formatRevenueSplit(0, 10000);
      expect(split.seatPercent).toBe(0);
      expect(split.executorPercent).toBe(100);
    });

    it('percentages always sum to 100 when there is revenue', () => {
      const split = formatRevenueSplit(3333, 6667);
      expect(split.seatPercent + split.executorPercent).toBe(100);
    });
  });

  // ===========================================================================
  // calculateGrowthRate
  // ===========================================================================
  describe('calculateGrowthRate', () => {
    it('calculates positive growth', () => {
      expect(calculateGrowthRate(112, 100)).toBeCloseTo(12);
    });

    it('calculates negative growth', () => {
      expect(calculateGrowthRate(90, 100)).toBeCloseTo(-10);
    });

    it('returns 0 when both are zero', () => {
      expect(calculateGrowthRate(0, 0)).toBe(0);
    });

    it('returns 100 when previous is zero and current is positive', () => {
      expect(calculateGrowthRate(50, 0)).toBe(100);
    });

    it('returns 0 when current is zero and previous is zero', () => {
      expect(calculateGrowthRate(0, 0)).toBe(0);
    });

    it('calculates 100% growth (doubling)', () => {
      expect(calculateGrowthRate(200, 100)).toBeCloseTo(100);
    });
  });

  // ===========================================================================
  // formatGrowthRate
  // ===========================================================================
  describe('formatGrowthRate', () => {
    it('formats positive rate with + prefix', () => {
      expect(formatGrowthRate(12.5)).toBe('+12.5%');
    });

    it('formats negative rate with - prefix', () => {
      expect(formatGrowthRate(-3.2)).toBe('-3.2%');
    });

    it('formats zero rate without prefix', () => {
      expect(formatGrowthRate(0)).toBe('0.0%');
    });

    it('formats large positive rate', () => {
      expect(formatGrowthRate(150)).toBe('+150.0%');
    });

    it('formats small negative rate', () => {
      expect(formatGrowthRate(-0.5)).toBe('-0.5%');
    });
  });
});
