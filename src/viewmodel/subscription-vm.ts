/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Subscription ViewModel
 *
 * Pure logic for the subscription & plan UI: plan metadata, upgrade/downgrade
 * checks, seat usage formatting, and payment display. No framework dependencies
 * -- consumed by SolidJS route components via simple function calls.
 */

// =============================================================================
// Types
// =============================================================================

/** Available plan tiers. */
export type PlanTier = 'free' | 'team' | 'enterprise';

/** Metadata for a single plan tier. */
export interface PlanDetails {
  name: string;
  price: string;
  pricePerSeat: number | null;
  features: string[];
  highlighted: boolean;
}

// =============================================================================
// Plan definitions
// =============================================================================

/** Plan metadata for display in the plan comparison section. */
export const PLANS: Record<PlanTier, PlanDetails> = {
  free: {
    name: 'Free',
    price: '$0/month',
    pricePerSeat: null,
    features: ['Up to 2 seats', 'BYO executors', '100 tasks/month', 'Community support'],
    highlighted: false,
  },
  team: {
    name: 'Team',
    price: '$15/seat/month',
    pricePerSeat: 15,
    features: [
      'Unlimited seats',
      'Managed executors',
      'Unlimited tasks',
      'RBAC & audit logs',
      'Priority support',
    ],
    highlighted: true,
  },
  enterprise: {
    name: 'Enterprise',
    price: 'Custom',
    pricePerSeat: null,
    features: [
      'Everything in Team',
      'Dedicated executors',
      'SSO (SAML/OIDC)',
      'SLA guarantee',
      'Custom integrations',
    ],
    highlighted: false,
  },
};

// =============================================================================
// Plan tier ordering (internal)
// =============================================================================

const TIER_ORDER: Record<PlanTier, number> = { free: 0, team: 1, enterprise: 2 };

// =============================================================================
// Upgrade / downgrade logic
// =============================================================================

/**
 * Whether the user can upgrade from currentPlan to targetPlan via self-serve.
 * Only free -> team is a self-serve upgrade; enterprise requires contact.
 */
export function canUpgrade(currentPlan: PlanTier, targetPlan: PlanTier): boolean {
  return currentPlan === 'free' && targetPlan === 'team';
}

/**
 * Whether the user can downgrade from currentPlan to targetPlan.
 * Downgrading is moving to a lower tier.
 */
export function canDowngrade(currentPlan: PlanTier, targetPlan: PlanTier): boolean {
  return TIER_ORDER[currentPlan] > TIER_ORDER[targetPlan];
}

/**
 * Returns the action label for a plan card button.
 */
export function getUpgradeAction(
  currentPlan: PlanTier,
  targetPlan: PlanTier,
): 'upgrade' | 'downgrade' | 'current' | 'contact' {
  if (currentPlan === targetPlan) {
    return 'current';
  }
  if (targetPlan === 'enterprise') {
    return 'contact';
  }
  if (TIER_ORDER[targetPlan] > TIER_ORDER[currentPlan]) {
    return 'upgrade';
  }
  return 'downgrade';
}

// =============================================================================
// Seat usage
// =============================================================================

/**
 * Formats seat usage for display.
 * @example "3 / 10 seats"
 */
export function formatSeatUsage(used: number, limit: number): string {
  return `${used} / ${limit} seats`;
}

/**
 * Returns seat usage as a percentage (0-100).
 */
export function getSeatUsagePercent(used: number, limit: number): number {
  if (limit <= 0) return 0;
  return Math.round((used / limit) * 100);
}

// =============================================================================
// Date formatting
// =============================================================================

/**
 * Formats a billing period end date for display.
 * @example "Renews on Apr 19, 2026"
 */
export function formatPeriodEnd(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    const formatted = date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
    return `Renews on ${formatted}`;
  } catch {
    return `Renews on ${dateStr}`;
  }
}

// =============================================================================
// Payment method formatting
// =============================================================================

/**
 * Formats a card for display.
 * @example "Visa ending in 4242"
 */
export function formatCardDisplay(brand: string, last4: string): string {
  const capitalizedBrand = brand.charAt(0).toUpperCase() + brand.slice(1);
  return `${capitalizedBrand} ending in ${last4}`;
}
