/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Organization Settings Page.
 *
 * Displays org info (name, slug, plan, created date) with an editable
 * name field and a danger zone for deleting the organization.
 * The component uses hardcoded mock data (Acme Corp, team plan).
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import SettingsRoute from '~/routes/portal/settings';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/SettingsPage',
  component: SettingsRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof SettingsRoute>;

type Story = StoryObj<typeof SettingsRoute>;

/**
 * Default view showing organization information section
 * and danger zone with delete button.
 */
export const Default: Story = {};

/**
 * Editing name: the org name input has been changed, showing the Save
 * button as enabled (not disabled). Changes the name field content.
 */
export const EditingName: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#org-name') as HTMLInputElement | null;
        if (nameInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            nameInput,
            'Acme Corporation (Renamed)',
          );
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Save success: shows the green "Organization name updated" message.
 * Changes the name and submits the form to trigger the success feedback.
 */
export const SaveSuccess: Story = {
  decorators: [
    withMockApi({
      '/api/v1/orgs': {
        id: 'org-1',
        name: 'Acme Corp Updated',
        slug: 'acme-corp',
        plan: 'team',
        createdAt: '2025-01-15T10:00:00Z',
      },
    }),
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#org-name') as HTMLInputElement | null;
        if (nameInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            nameInput,
            'Acme Corp Updated',
          );
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          const submitBtn = document.querySelector(
            'button[type="submit"]',
          ) as HTMLButtonElement | null;
          submitBtn?.click();
        }, 100);
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Delete modal open: the delete confirmation dialog is visible.
 * Clicks the "Delete Organization" button in the danger zone.
 */
export const DeleteModalOpen: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Delete Organization') {
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
 * Delete modal with slug partially typed in the confirmation input.
 * The "Delete Organization" button remains disabled until the slug matches.
 */
export const DeleteModalWithInput: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Open modal
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Delete Organization') {
            (btn as HTMLButtonElement).click();
            break;
          }
        }
        // Type partial slug
        setTimeout(() => {
          const slugInput = document.querySelector(
            '.fixed input[type="text"]',
          ) as HTMLInputElement | null;
          if (slugInput) {
            Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
              slugInput,
              'acme',
            );
            slugInput.dispatchEvent(new Event('input', { bubbles: true }));
          }
        }, 200);
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Delete modal ready: slug fully typed in confirmation, delete button enabled.
 * The full slug "acme-corp" matches the org slug, enabling the red delete button.
 */
export const DeleteModalReady: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Open modal
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Delete Organization') {
            (btn as HTMLButtonElement).click();
            break;
          }
        }
        // Type full slug
        setTimeout(() => {
          const slugInput = document.querySelector(
            '.fixed input[type="text"]',
          ) as HTMLInputElement | null;
          if (slugInput) {
            Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
              slugInput,
              'acme-corp',
            );
            slugInput.dispatchEvent(new Event('input', { bubbles: true }));
          }
        }, 200);
      }, 200);
      return <Story />;
    },
  ],
};
