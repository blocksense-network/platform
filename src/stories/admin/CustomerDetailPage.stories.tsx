/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Admin Customer Detail Page.
 *
 * Shows organization info, members, subscription details, recent invoices,
 * and admin actions (suspend/unsuspend) for a specific customer.
 * The component uses hardcoded mock data for Acme Corp.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import CustomerDetailsRoute from '~/routes/admin/customers/[orgId]';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Admin/CustomerDetailPage',
  component: CustomerDetailsRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof CustomerDetailsRoute>;

type Story = StoryObj<typeof CustomerDetailsRoute>;

/**
 * Full customer detail view with org info card, members table,
 * subscription section, and recent invoices table.
 * Shows Acme Corp on Team plan, active status, 5 seats, $75 MRR.
 */
export const Default: Story = {};

/**
 * Free plan customer: in production, would show no subscription section
 * (or a free plan indicator), no invoices, and $0 MRR.
 * The mock data shows Team plan; this documents the expected free plan state.
 */
export const FreePlanCustomer: Story = {};

/**
 * Suspended customer: the mock data shows active status with a "Suspend" button.
 * In production, a suspended customer would show a red "Suspended" badge and
 * an "Unsuspend" button (green) instead. The button text toggles based on status.
 */
export const SuspendedCustomer: Story = {};

/**
 * Customer with many members: the mock data includes 3 members.
 * In production, a customer with 10+ members would show a longer
 * table with scroll. This documents the multi-member table layout.
 */
export const ManyMembers: Story = {};

/**
 * Customer with no invoice history: the mock data includes 2 paid invoices.
 * When the recentInvoices array is empty, the "No invoices yet" fallback
 * text appears in the invoices section.
 */
export const NoInvoices: Story = {};
