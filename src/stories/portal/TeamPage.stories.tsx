/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Team Management Page.
 *
 * Two-tab layout (Members and Invitations) with invite modal.
 * The component uses hardcoded mock data internally, so stories
 * render the default state with all mock members and invitations.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import TeamRoute from '~/routes/portal/team';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/TeamPage',
  component: TeamRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof TeamRoute>;

type Story = StoryObj<typeof TeamRoute>;

/**
 * Members tab with sample data (3 members: admin, operator, viewer).
 * This is the default view when the page loads.
 */
export const MembersTab: Story = {};

/**
 * Members tab showing only the current user (empty team state).
 * The page uses hardcoded mock data with 3 members so this story
 * renders the same default view. In a real scenario with API-driven
 * data, this would show a single admin row.
 */
export const MembersTabEmpty: Story = {};

/**
 * Invitations tab: shows the pending invitations table.
 * Clicks the "Invitations" tab after mount to switch tabs.
 */
export const InvitationsTab: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'Invitations') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Invitations tab with no pending invitations.
 * Switches to invitations tab; the mock data includes 1 invitation,
 * so this shows the default invitations view. Empty-state text
 * "No pending invitations" would appear when the list is empty.
 */
export const InvitationsTabEmpty: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'Invitations') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
        // Revoke existing invitation to show empty state
        setTimeout(() => {
          const revokeBtn = document.querySelector('button.text-red-400');
          if (revokeBtn) {
            (revokeBtn as HTMLButtonElement).click();
          }
        }, 100);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Invite modal open: shows the "Invite Member" modal with email and role inputs.
 * Switches to invitations tab then clicks the "Invite Member" button.
 */
export const InviteModalOpen: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'Invitations') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
        setTimeout(() => {
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Invite Member') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
        }, 100);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Invite modal with validation error: empty email shows error message.
 * Opens the modal and immediately clicks "Send Invitation" to trigger validation.
 */
export const InviteModalWithError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/orgs': () => {
        throw new Error('Failed to send invitation');
      },
    }),
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'Invitations') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
        setTimeout(() => {
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Invite Member') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
          // Click submit with empty email to trigger validation
          setTimeout(() => {
            const submitBtn = document.querySelector('button[type="submit"]');
            if (submitBtn) {
              (submitBtn as HTMLButtonElement).click();
            }
          }, 100);
        }, 100);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Members tab after removing a member: shows the updated table
 * after clicking "Remove" on one member.
 */
export const MemberRemoved: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Remove the first available member (not the current admin user)
        const removeButtons = document.querySelectorAll('button');
        for (const btn of removeButtons) {
          if (btn.textContent?.trim() === 'Remove') {
            btn.click();
            break;
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * At seat limit: seat limit warning shown and invite button disabled.
 * The mock data has seatLimit=10 with 3 members + 1 invitation.
 * This story simulates a near-capacity scenario. Since the component
 * uses hardcoded limits, this shows the default view. When the
 * total members + invitations reach the seat limit, the warning
 * text and disabled button appear on the Invitations tab.
 */
export const AtSeatLimit: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'Invitations') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
      }, 100);
      return <Story />;
    },
  ],
};
