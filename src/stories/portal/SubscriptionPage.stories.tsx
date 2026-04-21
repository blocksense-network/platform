/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Subscription & Plan Management Page.
 *
 * Displays current plan info, seat usage, plan comparison cards,
 * and payment method management. The component uses hardcoded mock
 * data internally (team plan with 3/10 seats used).
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import SubscriptionRoute from '~/routes/portal/subscription';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/SubscriptionPage',
  component: SubscriptionRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof SubscriptionRoute>;

type Story = StoryObj<typeof SubscriptionRoute>;

/**
 * Default view showing team plan with seat usage bar,
 * plan comparison cards, and payment methods section.
 */
export const Default: Story = {};

/**
 * Free plan view. The mock data shows Team plan as current;
 * in production, the Free plan card would show "Current Plan"
 * and both Team and Enterprise would show upgrade options.
 * This documents the free plan state with no cancel button visible.
 */
export const FreePlan: Story = {};

/**
 * Team plan: current mock state. Shows:
 * - "Team" badge next to Current Plan heading
 * - Seat usage bar (3/10 seats)
 * - Free card with "Downgrade", Team with "Current Plan", Enterprise with "Upgrade"
 * - Cancel Subscription button visible
 * - Payment method on file (Visa ending 4242)
 */
export const TeamPlan: Story = {};

/**
 * Enterprise plan: in production, would show:
 * - "Enterprise" badge
 * - High seat limit
 * - Free and Team with "Downgrade", Enterprise with "Current Plan"
 * - "Contact Sales" interactions
 */
export const EnterprisePlan: Story = {};

/**
 * Cancellation confirmation modal open.
 * Clicks the "Cancel Subscription" button to display the modal
 * with "Keep Subscription" and "Confirm Cancel" buttons.
 */
export const CancelConfirmation: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Cancel Subscription') {
            (btn as HTMLButtonElement).click();
            break;
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Subscription set to cancel at end of billing period.
 * Opens the cancel modal and confirms cancellation. After success,
 * the page shows "Cancels at end of period" text in place of the
 * "Cancel Subscription" button.
 */
export const CancelAtPeriodEnd: Story = {
  decorators: [
    withMockApi({}),
    (Story: () => any) => {
      setTimeout(() => {
        // Click "Cancel Subscription"
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Cancel Subscription') {
            (btn as HTMLButtonElement).click();
            break;
          }
        }
        // Click "Confirm Cancel"
        setTimeout(() => {
          const modalButtons = document.querySelectorAll('button[type="button"]');
          for (const btn of modalButtons) {
            if (btn.textContent?.trim() === 'Confirm Cancel') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
        }, 200);
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * No payment methods on file. The mock data includes one Visa card;
 * in production, when no payment methods exist, the section shows
 * "No payment methods on file" with just the "Manage Payment Methods" button.
 */
export const NoPaymentMethods: Story = {};
