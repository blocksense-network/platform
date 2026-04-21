/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Admin Revenue Dashboard Page.
 *
 * Displays aggregate revenue metrics including MRR, customer counts,
 * revenue split between seats and executors, and breakdown by plan tier.
 * The component uses hardcoded mock data:
 * - MRR: $695.00 (up from $580.00 previous month = +19.8% growth)
 * - 12 total customers (8 paid, 4 free)
 * - 15 active executors
 * - Seat revenue: $450.00 (64.7%), Executor revenue: $245.00 (35.3%)
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import RevenueDashboardRoute from '~/routes/admin/revenue';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Admin/RevenuePage',
  component: RevenueDashboardRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof RevenueDashboardRoute>;

type Story = StoryObj<typeof RevenueDashboardRoute>;

/**
 * Revenue dashboard with MRR metric card (including growth rate),
 * customer/executor counts, revenue split bars, and plan tier breakdown table.
 */
export const Default: Story = {};

/**
 * High growth state: the default mock data shows +19.8% growth
 * (from $580 to $695 MRR). The growth rate appears as a green
 * percentage next to the MRR value.
 */
export const HighGrowth: Story = {};

/**
 * Declining state: in production, a negative growth rate would show
 * a red percentage (e.g., -5.0%). The mock data shows positive growth,
 * so this documents the negative growth appearance.
 */
export const Declining: Story = {};

/**
 * All seat revenue: in production, when executor revenue is $0,
 * the seat revenue bar would be at 100% and the executor bar at 0%.
 * The mock data shows a 65/35 split.
 */
export const AllSeatRevenue: Story = {};

/**
 * Mixed revenue split: the default mock shows approximately 65% seats
 * ($450) and 35% executors ($245). Both progress bars are partially filled.
 */
export const MixedRevenue: Story = {};

/**
 * Zero revenue state: in production with a fresh deployment (no customers),
 * all metrics would show $0.00 MRR, 0 customers, 0 executors, and empty
 * plan tier breakdown. The growth rate would show +0.0%.
 */
export const ZeroRevenue: Story = {};
