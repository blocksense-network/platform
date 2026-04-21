/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Admin Customers ViewModel
 *
 * Pure logic for the admin customer management UI: filtering, sorting,
 * formatting, and status display. No framework dependencies -- consumed
 * by SolidJS route components via simple function calls.
 */

import type { AdminCustomer } from '@agent-harbor/rest-client';

// =============================================================================
// Filtering
// =============================================================================

/**
 * Filters customers by search text (matches orgName) and plan tier.
 *
 * @param customers - Array of admin customers
 * @param search - Search string to match against orgName (case-insensitive)
 * @param planFilter - Plan tier to filter by, or null for all plans
 * @returns Filtered array of customers
 */
export function filterCustomers(
  customers: AdminCustomer[],
  search: string,
  planFilter: string | null,
): AdminCustomer[] {
  const lowerSearch = search.toLowerCase().trim();

  return customers.filter(c => {
    if (lowerSearch && !c.orgName.toLowerCase().includes(lowerSearch)) {
      return false;
    }
    if (planFilter && c.plan !== planFilter) {
      return false;
    }
    return true;
  });
}

// =============================================================================
// Sorting
// =============================================================================

/** Sort key options for the customer table. */
export type CustomerSortKey = 'name' | 'mrr' | 'created';

/** Sort direction. */
export type SortDirection = 'asc' | 'desc';

/**
 * Sorts customers by the given key and direction.
 *
 * @param customers - Array of admin customers
 * @param sortBy - Sort key: 'name', 'mrr', or 'created'
 * @param direction - Sort direction: 'asc' or 'desc'
 * @returns New sorted array
 */
export function sortCustomers(
  customers: AdminCustomer[],
  sortBy: CustomerSortKey,
  direction: SortDirection,
): AdminCustomer[] {
  const sorted = [...customers];
  const mult = direction === 'asc' ? 1 : -1;

  sorted.sort((a, b) => {
    switch (sortBy) {
      case 'name':
        return mult * a.orgName.localeCompare(b.orgName);
      case 'mrr':
        return mult * (a.mrr - b.mrr);
      case 'created':
        return mult * (new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());
      default:
        return 0;
    }
  });

  return sorted;
}

// =============================================================================
// Formatting
// =============================================================================

/**
 * Formats a monthly recurring revenue value (in cents) for display.
 *
 * @example formatMrr(15000) => "$150.00/mo"
 * @example formatMrr(0) => "$0.00/mo"
 */
export function formatMrr(cents: number): string {
  const amount = cents / 100;
  return `$${amount.toFixed(2)}/mo`;
}

// =============================================================================
// Status display
// =============================================================================

/** Badge display properties for a customer status. */
export interface StatusBadge {
  color: string;
  label: string;
}

/**
 * Returns badge color and label for a customer status.
 *
 * @param status - Customer status string
 * @returns Badge with color name and human-readable label
 */
export function getCustomerStatusBadge(status: string): StatusBadge {
  switch (status) {
    case 'active':
      return { color: 'green', label: 'Active' };
    case 'suspended':
      return { color: 'yellow', label: 'Suspended' };
    case 'deleted':
      return { color: 'red', label: 'Deleted' };
    default:
      return { color: 'gray', label: status };
  }
}
