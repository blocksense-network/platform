/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Accept Invitation Page.
 *
 * Three states: loading (fetching invitation), valid invitation (org name and
 * sign-in/sign-up options), and expired/invalid invitation error.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import AcceptInviteRoute from '~/routes/auth/invite/[token]';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Auth/InviteAcceptPage',
  component: AcceptInviteRoute,
  decorators: [withPageLayout],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof AcceptInviteRoute>;

type Story = StoryObj<typeof AcceptInviteRoute>;

/**
 * Loading state: fetching invitation details (spinner visible).
 * The mock endpoint never resolves, keeping the component in loading state.
 */
export const Loading: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/invitations/': () => new Promise(() => {}), // never resolves
    }),
  ],
};

/**
 * Valid invitation: shows the organization name, inviter, and options
 * to sign in or create a new account.
 */
export const ValidInvitation: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/invitations/': {
        organizationName: 'Acme Corp',
        inviterName: 'Alice Admin',
        email: 'invited@example.com',
      },
    }),
  ],
};

/**
 * Expired invitation: shows an error message when the invitation
 * token is invalid or has expired.
 */
export const ExpiredInvitation: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/invitations/': () => {
        throw new Error('This invitation has expired or is no longer valid.');
      },
    }),
  ],
};
