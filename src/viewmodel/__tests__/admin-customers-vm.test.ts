/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Admin Customers ViewModel Tests (M12)
 */

import { describe, it, expect } from 'vitest';
import {
  filterCustomers,
  sortCustomers,
  formatMrr,
  getCustomerStatusBadge,
} from '../admin-customers-vm';
import type { AdminCustomer } from '@agent-harbor/rest-client';

// =============================================================================
// Test data
// =============================================================================

const MOCK_CUSTOMERS: AdminCustomer[] = [
  {
    orgId: 'org-1',
    orgName: 'Acme Corp',
    slug: 'acme-corp',
    plan: 'team',
    seatsUsed: 5,
    mrr: 7500,
    executorSpend: 2000,
    status: 'active',
    createdAt: '2025-01-15T10:00:00Z',
  },
  {
    orgId: 'org-2',
    orgName: 'Beta Industries',
    slug: 'beta-industries',
    plan: 'enterprise',
    seatsUsed: 20,
    mrr: 50000,
    executorSpend: 15000,
    status: 'active',
    createdAt: '2025-03-10T09:00:00Z',
  },
  {
    orgId: 'org-3',
    orgName: 'Gamma Labs',
    slug: 'gamma-labs',
    plan: 'free',
    seatsUsed: 2,
    mrr: 0,
    executorSpend: 0,
    status: 'suspended',
    createdAt: '2025-02-20T14:00:00Z',
  },
];

describe('Admin Customers ViewModel', () => {
  // ===========================================================================
  // filterCustomers
  // ===========================================================================
  describe('filterCustomers', () => {
    it('returns all customers when no filters applied', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, '', null);
      expect(result).toHaveLength(3);
    });

    it('filters by search term matching orgName', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, 'acme', null);
      expect(result).toHaveLength(1);
      expect(result[0].orgId).toBe('org-1');
    });

    it('search is case-insensitive', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, 'BETA', null);
      expect(result).toHaveLength(1);
      expect(result[0].orgId).toBe('org-2');
    });

    it('filters by plan', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, '', 'free');
      expect(result).toHaveLength(1);
      expect(result[0].orgId).toBe('org-3');
    });

    it('combines search and plan filter', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, 'a', 'team');
      expect(result).toHaveLength(1);
      expect(result[0].orgId).toBe('org-1');
    });

    it('returns empty for no matches', () => {
      const result = filterCustomers(MOCK_CUSTOMERS, 'nonexistent', null);
      expect(result).toHaveLength(0);
    });
  });

  // ===========================================================================
  // sortCustomers
  // ===========================================================================
  describe('sortCustomers', () => {
    it('sorts by name ascending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'name', 'asc');
      expect(result[0].orgName).toBe('Acme Corp');
      expect(result[1].orgName).toBe('Beta Industries');
      expect(result[2].orgName).toBe('Gamma Labs');
    });

    it('sorts by name descending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'name', 'desc');
      expect(result[0].orgName).toBe('Gamma Labs');
      expect(result[2].orgName).toBe('Acme Corp');
    });

    it('sorts by mrr ascending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'mrr', 'asc');
      expect(result[0].mrr).toBe(0);
      expect(result[1].mrr).toBe(7500);
      expect(result[2].mrr).toBe(50000);
    });

    it('sorts by mrr descending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'mrr', 'desc');
      expect(result[0].mrr).toBe(50000);
      expect(result[2].mrr).toBe(0);
    });

    it('sorts by created ascending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'created', 'asc');
      expect(result[0].orgId).toBe('org-1');
      expect(result[1].orgId).toBe('org-3');
      expect(result[2].orgId).toBe('org-2');
    });

    it('sorts by created descending', () => {
      const result = sortCustomers(MOCK_CUSTOMERS, 'created', 'desc');
      expect(result[0].orgId).toBe('org-2');
      expect(result[2].orgId).toBe('org-1');
    });

    it('does not mutate the original array', () => {
      const original = [...MOCK_CUSTOMERS];
      sortCustomers(MOCK_CUSTOMERS, 'mrr', 'desc');
      expect(MOCK_CUSTOMERS).toEqual(original);
    });
  });

  // ===========================================================================
  // formatMrr
  // ===========================================================================
  describe('formatMrr', () => {
    it('formats zero cents', () => {
      expect(formatMrr(0)).toBe('$0.00/mo');
    });

    it('formats 15000 cents as $150.00/mo', () => {
      expect(formatMrr(15000)).toBe('$150.00/mo');
    });

    it('formats 7500 cents as $75.00/mo', () => {
      expect(formatMrr(7500)).toBe('$75.00/mo');
    });

    it('formats 99 cents as $0.99/mo', () => {
      expect(formatMrr(99)).toBe('$0.99/mo');
    });
  });

  // ===========================================================================
  // getCustomerStatusBadge
  // ===========================================================================
  describe('getCustomerStatusBadge', () => {
    it('returns green for active', () => {
      const badge = getCustomerStatusBadge('active');
      expect(badge.color).toBe('green');
      expect(badge.label).toBe('Active');
    });

    it('returns yellow for suspended', () => {
      const badge = getCustomerStatusBadge('suspended');
      expect(badge.color).toBe('yellow');
      expect(badge.label).toBe('Suspended');
    });

    it('returns red for deleted', () => {
      const badge = getCustomerStatusBadge('deleted');
      expect(badge.color).toBe('red');
      expect(badge.label).toBe('Deleted');
    });

    it('returns gray for unknown status', () => {
      const badge = getCustomerStatusBadge('unknown');
      expect(badge.color).toBe('gray');
      expect(badge.label).toBe('unknown');
    });
  });
});
