/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Admin Executor Fleet Page.
 *
 * Displays the full fleet of managed executors across all organizations
 * with filtering by status and machine class, plus summary statistics.
 * The component uses hardcoded mock data with 5 executors in mixed statuses.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import ExecutorFleetRoute from '~/routes/admin/executors';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Admin/ExecutorFleetPage',
  component: ExecutorFleetRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof ExecutorFleetRoute>;

type Story = StoryObj<typeof ExecutorFleetRoute>;

/**
 * Fleet overview with mixed statuses (running, provisioning, stopped).
 * Shows summary stats, filter dropdowns, and the fleet table.
 */
export const Default: Story = {};

/**
 * Empty fleet: no executors match the filter criteria.
 * Selects "terminated" status filter which has no matches in mock data,
 * showing "No executors found" empty state.
 */
export const EmptyFleet: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const selects = document.querySelectorAll('[data-testid="fleet-filters"] select');
        if (selects.length > 0) {
          const statusSelect = selects[0] as HTMLSelectElement;
          // Find the "terminated" option (not present in mock data, so shows empty)
          for (const opt of statusSelect.options) {
            if (opt.value === 'terminated') {
              statusSelect.value = 'terminated';
              statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
              break;
            }
          }
          // If no terminated option exists, pick a non-existent status
          // to force the empty state by typing in the select
          if (statusSelect.value !== 'terminated') {
            // Alternative: filter by a class that has no executors
            // Use class filter with a value that won't match
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Filtered by running status: shows only the 3 running executors.
 * Selects "running" in the status filter dropdown.
 */
export const FilteredByRunning: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const selects = document.querySelectorAll('[data-testid="fleet-filters"] select');
        if (selects.length > 0) {
          const statusSelect = selects[0] as HTMLSelectElement;
          for (const opt of statusSelect.options) {
            if (opt.value === 'running') {
              statusSelect.value = 'running';
              statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
              break;
            }
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Filtered by large (8 vCPU) machine class: shows only large executors.
 * Selects "large (8 vCPU)" in the machine class filter dropdown.
 */
export const FilteredByGPU: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const selects = document.querySelectorAll('[data-testid="fleet-filters"] select');
        if (selects.length > 1) {
          const classSelect = selects[1] as HTMLSelectElement;
          for (const opt of classSelect.options) {
            if (opt.value.includes('large')) {
              classSelect.value = opt.value;
              classSelect.dispatchEvent(new Event('change', { bubbles: true }));
              break;
            }
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Filtered by stopped status: shows only stopped executors.
 */
export const FilteredByStopped: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const selects = document.querySelectorAll('[data-testid="fleet-filters"] select');
        if (selects.length > 0) {
          const statusSelect = selects[0] as HTMLSelectElement;
          for (const opt of statusSelect.options) {
            if (opt.value === 'stopped') {
              statusSelect.value = 'stopped';
              statusSelect.dispatchEvent(new Event('change', { bubbles: true }));
              break;
            }
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Filtered by small machine class: shows only small (2 vCPU) executors.
 */
export const FilteredBySmall: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const selects = document.querySelectorAll('[data-testid="fleet-filters"] select');
        if (selects.length > 1) {
          const classSelect = selects[1] as HTMLSelectElement;
          for (const opt of classSelect.options) {
            if (opt.value.includes('small')) {
              classSelect.value = opt.value;
              classSelect.dispatchEvent(new Event('change', { bubbles: true }));
              break;
            }
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * High utilization view: all executors visible with summary stats.
 * The default mock shows 3 active (running) out of 5 total executors.
 * Summary cards show: Total Active (3), Total Executors (5),
 * and by-class breakdown.
 */
export const HighUtilization: Story = {};
