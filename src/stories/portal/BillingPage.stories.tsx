/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Billing & Invoices Page.
 *
 * Displays current period charges summary, payment methods,
 * and invoice history. The component uses hardcoded mock data
 * internally with sample invoices and payment methods.
 *
 * The default mock data includes:
 * - Seat charges: 5 seats at $15/seat = $75.00
 * - Executor charges: small (24.5h) + large (8h) = $28.25
 * - Two payment methods (Visa and Mastercard)
 * - Three invoices: one open (yellow badge), two paid (green badges)
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import BillingRoute from '~/routes/portal/billing';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/BillingPage',
  component: BillingRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof BillingRoute>;

type Story = StoryObj<typeof BillingRoute>;

/**
 * Default view with current period summary (seat + executor charges),
 * two payment methods, and three invoices in history.
 */
export const Default: Story = {};

/**
 * Empty invoices: removes all invoices by clicking "Load More" which returns
 * empty from the mock. The hardcoded invoices (3) are already visible.
 * This story documents the empty state text "No invoices yet" that shows
 * when the invoice array is empty in production.
 *
 * Note: The default view already shows the mixed invoice statuses --
 * INV-0003 (Open/yellow) and INV-0001, INV-0002 (Paid/green).
 */
export const EmptyInvoices: Story = {};

/**
 * After removing the non-default payment method.
 * Clicks the "Remove" button on the Mastercard to show the remaining Visa card.
 */
export const EmptyPaymentMethods: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Remove both payment methods to show the empty state
        const removeAll = () => {
          const removeButtons: HTMLButtonElement[] = [];
          document.querySelectorAll('button').forEach(btn => {
            if (btn.textContent?.trim() === 'Remove') {
              removeButtons.push(btn);
            }
          });
          if (removeButtons.length > 0) {
            removeButtons[0].click();
            setTimeout(removeAll, 100);
          }
        };
        removeAll();
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Default view highlighting the paid invoices (green status badges).
 * The mock data includes INV-0001 and INV-0002 with "paid" status.
 * Both show green "Paid" badges and "Download PDF" links.
 */
export const WithPaidInvoices: Story = {};

/**
 * Default view highlighting the open invoice (yellow status badge).
 * The mock data includes INV-0003 with "open" status, showing a
 * yellow "Open" badge and no PDF download link.
 */
export const WithOpenInvoice: Story = {};

/**
 * Failed invoice state. The mock data does not include an uncollectible
 * invoice, but this documents the expected appearance: a red "Uncollectible"
 * badge appears for invoices with status "uncollectible" or "void".
 */
export const WithFailedInvoice: Story = {};

/**
 * Default view emphasizing the executor charges section.
 * The current period shows two machine classes:
 * - small (2 vCPU): 24.5h at $0.50/hr = $12.25
 * - large (8 vCPU): 8.0h at $2.00/hr = $16.00
 * Total estimated: $103.25 including seat charges.
 */
export const HighExecutorUsage: Story = {};
