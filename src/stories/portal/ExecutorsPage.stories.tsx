/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Managed Executors Page.
 *
 * Two-tab layout (Active Executors, History) with provision modal.
 * The component uses hardcoded mock data with executors in various
 * statuses: running, provisioning, stopped, and terminated.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import ExecutorsRoute from '~/routes/portal/executors';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/ExecutorsPage',
  component: ExecutorsRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof ExecutorsRoute>;

type Story = StoryObj<typeof ExecutorsRoute>;

/**
 * Active executors tab with mixed statuses (running, provisioning, stopped).
 * Shows name, machine class, OS, region, status badge, uptime, and cost.
 */
export const Default: Story = {};

/**
 * Empty active executors state: terminates all active executors to show
 * the "No active executors" message. Clicks Terminate on each executor.
 */
export const EmptyState: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const terminateAll = () => {
          const buttons: HTMLButtonElement[] = [];
          document.querySelectorAll('button').forEach(btn => {
            if (btn.textContent?.trim() === 'Terminate') {
              buttons.push(btn);
            }
          });
          if (buttons.length > 0) {
            buttons[0].click();
            setTimeout(terminateAll, 100);
          }
        };
        terminateAll();
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * All executor statuses visible: the default mock data includes
 * running (exec-1), provisioning (exec-2), stopped (exec-3), and
 * terminated (exec-4 in history tab). This matches the default view.
 */
export const AllStatuses: Story = {};

/**
 * Provision modal open: shows the executor provisioning form with
 * machine class, OS, region dropdowns and optional name field.
 */
export const ProvisionModalOpen: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const btn = document.querySelector(
          '[data-testid="provision-button"]',
        ) as HTMLButtonElement | null;
        btn?.click();
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Provision modal with a machine class selected, showing the cost estimate.
 * Selects "Medium" machine class to reveal OS/region options and cost.
 */
export const ProvisionModalWithSelection: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const btn = document.querySelector(
          '[data-testid="provision-button"]',
        ) as HTMLButtonElement | null;
        btn?.click();
        setTimeout(() => {
          const select = document.querySelector(
            '#provision-machine-class',
          ) as HTMLSelectElement | null;
          if (select) {
            // Select the second option (Medium)
            if (select.options.length > 2) {
              select.value = select.options[2].value;
              select.dispatchEvent(new Event('change', { bubbles: true }));
            }
            setTimeout(() => {
              // Select OS
              const osSelect = document.querySelector('#provision-os') as HTMLSelectElement | null;
              if (osSelect && osSelect.options.length > 1) {
                osSelect.value = osSelect.options[1].value;
                osSelect.dispatchEvent(new Event('change', { bubbles: true }));
              }
              // Select Region
              const regionSelect = document.querySelector(
                '#provision-region',
              ) as HTMLSelectElement | null;
              if (regionSelect && regionSelect.options.length > 1) {
                regionSelect.value = regionSelect.options[1].value;
                regionSelect.dispatchEvent(new Event('change', { bubbles: true }));
              }
            }, 100);
          }
        }, 200);
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * After stopping a running executor: clicks Stop on the first running
 * executor to change its status to "stopped".
 */
export const AfterStopAction: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const buttons = document.querySelectorAll('button');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Stop') {
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
 * History tab: shows terminated executors with total runtime and cost.
 * Clicks the "History" tab to switch views.
 */
export const HistoryTab: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tabs = document.querySelectorAll('button[type="button"]');
        for (const tab of tabs) {
          if (tab.textContent?.trim() === 'History') {
            (tab as HTMLButtonElement).click();
            break;
          }
        }
      }, 100);
      return <Story />;
    },
  ],
};
