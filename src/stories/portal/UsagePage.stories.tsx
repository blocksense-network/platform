/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Usage Dashboard Page.
 *
 * Two-tab layout (Control Plane, Executors) with date range filtering,
 * stats cards, and CSV export. The component uses hardcoded mock data
 * with randomly generated usage data points.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import PortalUsagePage from '~/routes/portal/usage';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Portal/UsagePage',
  component: PortalUsagePage,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof PortalUsagePage>;

type Story = StoryObj<typeof PortalUsagePage>;

/**
 * Control Plane tab with seat usage data, stats cards,
 * and the seat usage bar chart (default view).
 */
export const Default: Story = {};

/**
 * Control Plane tab explicitly selected with stats cards showing
 * Current Seats, Average, and Peak values.
 */
export const ControlPlaneTab: Story = {};

/**
 * Executors tab: shows total hours, total cost stats cards, and
 * machine class breakdown table (small, medium, large).
 */
export const ExecutorsTab: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tab = document.querySelector(
          '[data-testid="tab-executors"]',
        ) as HTMLButtonElement | null;
        tab?.click();
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * 7-day date range: filters usage data to the last 7 days.
 * Shows a shorter bar chart and updated average/peak calculations.
 */
export const DateRange7Days: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const btn = document.querySelector('[data-testid="range-7d"]') as HTMLButtonElement | null;
        btn?.click();
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * 90-day date range: shows the full 90-day usage data set.
 * Displays more data points in the bar chart.
 */
export const DateRange90Days: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const btn = document.querySelector('[data-testid="range-90d"]') as HTMLButtonElement | null;
        btn?.click();
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Executors tab with 7-day range: shows executor usage data
 * filtered to the last 7 days.
 */
export const ExecutorsTab7Days: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const tab = document.querySelector(
          '[data-testid="tab-executors"]',
        ) as HTMLButtonElement | null;
        tab?.click();
        setTimeout(() => {
          const btn = document.querySelector(
            '[data-testid="range-7d"]',
          ) as HTMLButtonElement | null;
          btn?.click();
        }, 100);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Empty state: no usage data available yet.
 * The component always shows randomly generated mock data,
 * so this documents the expected appearance when the data
 * arrays are empty in production (stats show 0, no bars).
 */
export const EmptyState: Story = {};
