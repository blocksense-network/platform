/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Subscription ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  PLANS,
  canUpgrade,
  canDowngrade,
  getUpgradeAction,
  formatSeatUsage,
  getSeatUsagePercent,
  formatPeriodEnd,
  formatCardDisplay,
  type PlanTier,
} from '../subscription-vm';

describe('Subscription ViewModel', () => {
  // ===========================================================================
  // PLANS structure
  // ===========================================================================
  describe('PLANS', () => {
    it('has all three tiers', () => {
      expect(Object.keys(PLANS)).toEqual(['free', 'team', 'enterprise']);
    });

    it('free tier has correct structure', () => {
      expect(PLANS.free.name).toBe('Free');
      expect(PLANS.free.price).toBe('$0/month');
      expect(PLANS.free.pricePerSeat).toBeNull();
      expect(PLANS.free.highlighted).toBe(false);
      expect(PLANS.free.features.length).toBeGreaterThan(0);
    });

    it('team tier is highlighted', () => {
      expect(PLANS.team.name).toBe('Team');
      expect(PLANS.team.pricePerSeat).toBe(15);
      expect(PLANS.team.highlighted).toBe(true);
    });

    it('enterprise tier has correct structure', () => {
      expect(PLANS.enterprise.name).toBe('Enterprise');
      expect(PLANS.enterprise.price).toBe('Custom');
      expect(PLANS.enterprise.pricePerSeat).toBeNull();
      expect(PLANS.enterprise.highlighted).toBe(false);
    });
  });

  // ===========================================================================
  // canUpgrade
  // ===========================================================================
  describe('canUpgrade', () => {
    it('free -> team is true', () => {
      expect(canUpgrade('free', 'team')).toBe(true);
    });

    it('free -> enterprise is false (contact required)', () => {
      expect(canUpgrade('free', 'enterprise')).toBe(false);
    });

    it('team -> enterprise is false (contact required)', () => {
      expect(canUpgrade('team', 'enterprise')).toBe(false);
    });

    it('team -> free is false (that is downgrade)', () => {
      expect(canUpgrade('team', 'free')).toBe(false);
    });

    it('free -> free is false', () => {
      expect(canUpgrade('free', 'free')).toBe(false);
    });
  });

  // ===========================================================================
  // canDowngrade
  // ===========================================================================
  describe('canDowngrade', () => {
    it('team -> free is true', () => {
      expect(canDowngrade('team', 'free')).toBe(true);
    });

    it('enterprise -> team is true', () => {
      expect(canDowngrade('enterprise', 'team')).toBe(true);
    });

    it('enterprise -> free is true', () => {
      expect(canDowngrade('enterprise', 'free')).toBe(true);
    });

    it('free -> team is false', () => {
      expect(canDowngrade('free', 'team')).toBe(false);
    });

    it('free -> free is false', () => {
      expect(canDowngrade('free', 'free')).toBe(false);
    });
  });

  // ===========================================================================
  // getUpgradeAction
  // ===========================================================================
  describe('getUpgradeAction', () => {
    it('same plan returns current', () => {
      const tiers: PlanTier[] = ['free', 'team', 'enterprise'];
      for (const tier of tiers) {
        expect(getUpgradeAction(tier, tier)).toBe('current');
      }
    });

    it('free -> team returns upgrade', () => {
      expect(getUpgradeAction('free', 'team')).toBe('upgrade');
    });

    it('free -> enterprise returns contact', () => {
      expect(getUpgradeAction('free', 'enterprise')).toBe('contact');
    });

    it('team -> enterprise returns contact', () => {
      expect(getUpgradeAction('team', 'enterprise')).toBe('contact');
    });

    it('team -> free returns downgrade', () => {
      expect(getUpgradeAction('team', 'free')).toBe('downgrade');
    });

    it('enterprise -> team returns downgrade', () => {
      expect(getUpgradeAction('enterprise', 'team')).toBe('downgrade');
    });

    it('enterprise -> free returns downgrade', () => {
      expect(getUpgradeAction('enterprise', 'free')).toBe('downgrade');
    });
  });

  // ===========================================================================
  // formatSeatUsage
  // ===========================================================================
  describe('formatSeatUsage', () => {
    it('formats usage with limit', () => {
      expect(formatSeatUsage(3, 10)).toBe('3 / 10 seats');
    });

    it('formats at limit', () => {
      expect(formatSeatUsage(2, 2)).toBe('2 / 2 seats');
    });

    it('formats zero usage', () => {
      expect(formatSeatUsage(0, 10)).toBe('0 / 10 seats');
    });
  });

  // ===========================================================================
  // getSeatUsagePercent
  // ===========================================================================
  describe('getSeatUsagePercent', () => {
    it('returns 30 for 3/10', () => {
      expect(getSeatUsagePercent(3, 10)).toBe(30);
    });

    it('returns 100 for 2/2', () => {
      expect(getSeatUsagePercent(2, 2)).toBe(100);
    });

    it('returns 0 for 0/10', () => {
      expect(getSeatUsagePercent(0, 10)).toBe(0);
    });

    it('returns 0 for 0 limit', () => {
      expect(getSeatUsagePercent(5, 0)).toBe(0);
    });
  });

  // ===========================================================================
  // formatPeriodEnd
  // ===========================================================================
  describe('formatPeriodEnd', () => {
    it('formats a date string', () => {
      const result = formatPeriodEnd('2026-04-19T00:00:00Z');
      expect(result).toMatch(/Renews on Apr 19, 2026/);
    });
  });

  // ===========================================================================
  // formatCardDisplay
  // ===========================================================================
  describe('formatCardDisplay', () => {
    it('formats Visa card', () => {
      expect(formatCardDisplay('visa', '4242')).toBe('Visa ending in 4242');
    });

    it('formats Mastercard', () => {
      expect(formatCardDisplay('mastercard', '5555')).toBe('Mastercard ending in 5555');
    });

    it('capitalizes brand', () => {
      expect(formatCardDisplay('amex', '1234')).toBe('Amex ending in 1234');
    });
  });
});
