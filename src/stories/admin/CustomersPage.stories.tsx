/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Admin Customer Management Page.
 *
 * Displays a filterable, sortable table of all customer organizations
 * with plan, seats, MRR, executor spend, and status columns.
 * The component uses hardcoded mock data with 4 sample organizations.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import CustomersIndexRoute from '~/routes/admin/customers/index';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Admin/CustomersPage',
  component: CustomersIndexRoute,
  decorators: [withPageLayout, withMockApi({})],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof CustomersIndexRoute>;

type Story = StoryObj<typeof CustomersIndexRoute>;

/**
 * Customer list with sample data (4 orgs across free, team, enterprise plans).
 * Includes search input and plan filter dropdown.
 */
export const Default: Story = {};

/**
 * Filtered by Team plan: selects "team" in the plan filter dropdown
 * to show only Team plan customers (Acme Corp and Delta Dynamics).
 */
export const FilteredByTeamPlan: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const select = document.querySelector(
          '[data-testid="customer-filters"] select',
        ) as HTMLSelectElement | null;
        if (select) {
          select.value = 'team';
          select.dispatchEvent(new Event('change', { bubbles: true }));
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Search results: types a search query to filter by organization name.
 * Searches for "Beta" which matches "Beta Industries".
 */
export const SearchResults: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const searchInput = document.querySelector(
          '[data-testid="customer-filters"] input',
        ) as HTMLInputElement | null;
        if (searchInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            searchInput,
            'Beta',
          );
          searchInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * No customers match filter: searches for a non-existent name.
 * Shows the "No customers found" empty state row.
 */
export const EmptyResults: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const searchInput = document.querySelector(
          '[data-testid="customer-filters"] input',
        ) as HTMLInputElement | null;
        if (searchInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            searchInput,
            'Nonexistent Corp XYZ',
          );
          searchInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Filtered by Enterprise plan: shows only enterprise customers.
 * Selects "enterprise" in the plan filter dropdown.
 */
export const FilteredByEnterprisePlan: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const select = document.querySelector(
          '[data-testid="customer-filters"] select',
        ) as HTMLSelectElement | null;
        if (select) {
          select.value = 'enterprise';
          select.dispatchEvent(new Event('change', { bubbles: true }));
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Sorted by name ascending (default sort): org names appear alphabetically.
 */
export const SortedByName: Story = {};

/**
 * Sorted by creation date: clicks the "Created" column header.
 */
export const SortedByCreated: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const headers = document.querySelectorAll('[data-testid="customer-table"] th');
        for (const th of headers) {
          if (th.textContent?.includes('Created')) {
            (th as HTMLElement).click();
            break;
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};

/**
 * Sorted by MRR descending: clicks the MRR column header to sort.
 * Beta Industries ($500.00) appears first, followed by Delta Dynamics ($120.00),
 * then Acme Corp ($75.00), and Gamma Labs ($0.00).
 */
export const SortedByMrr: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Click MRR header to sort ascending, then again for descending
        const headers = document.querySelectorAll('[data-testid="customer-table"] th');
        for (const th of headers) {
          if (th.textContent?.includes('MRR')) {
            (th as HTMLElement).click(); // asc
            setTimeout(() => (th as HTMLElement).click(), 50); // desc
            break;
          }
        }
      }, 200);
      return <Story />;
    },
  ],
};
