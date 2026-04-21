/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 */

/**
 * Platform Navigation Items
 *
 * Exports the platform-specific nav sections that the nav-registry
 * will use when the @agent-harbor/platform package is available.
 * These are the canonical definitions; the nav-registry in the main
 * webui duplicates them as inline constants for bundle-size reasons.
 */

export interface PlatformNavItem {
  name: string;
  href: string;
  icon: string;
  exact?: boolean;
  requires?: string;
}

export interface PlatformNavSection {
  title: string | null;
  items: PlatformNavItem[];
  requires?: string;
}

/** Portal: Organization section (teams) */
export const PORTAL_ORGANIZATION_SECTION: PlatformNavSection = {
  title: 'Organization',
  requires: 'teams',
  items: [
    {
      name: 'Team',
      href: '/portal/team',
      icon: 'M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z',
      exact: true,
      requires: 'teams',
    },
    {
      name: 'Settings',
      href: '/portal/settings',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z',
      exact: true,
      requires: 'teams',
    },
  ],
};

/** Portal: Billing section */
export const PORTAL_BILLING_SECTION: PlatformNavSection = {
  title: 'Billing',
  requires: 'billing',
  items: [
    {
      name: 'Subscription',
      href: '/portal/subscription',
      icon: 'M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z',
      exact: true,
      requires: 'billing',
    },
    {
      name: 'Executors',
      href: '/portal/executors',
      icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01',
      exact: true,
      requires: 'managedExecutors',
    },
  ],
};

/** Admin: Platform section */
export const ADMIN_PLATFORM_SECTION: PlatformNavSection = {
  title: 'Platform',
  requires: 'billing',
  items: [
    {
      name: 'Customers',
      href: '/admin/customers',
      icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z',
      exact: false,
      requires: 'billing',
    },
    {
      name: 'Revenue',
      href: '/admin/revenue',
      icon: 'M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
      exact: true,
      requires: 'billing',
    },
    {
      name: 'Executors',
      href: '/admin/executors',
      icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01',
      exact: true,
      requires: 'managedExecutors',
    },
  ],
};
