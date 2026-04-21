/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Email Verification Page.
 *
 * Three states: loading (spinner), success (email verified), error (invalid/expired token).
 * The component reads ?token= from the URL and calls the verify endpoint on mount.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import VerifyEmailRoute from '~/routes/auth/verify-email';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Auth/VerifyEmailPage',
  component: VerifyEmailRoute,
  decorators: [withPageLayout],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof VerifyEmailRoute>;

type Story = StoryObj<typeof VerifyEmailRoute>;

/**
 * Loading state: verification in progress (spinner visible).
 * Mock endpoint never resolves, keeping the component in the loading state.
 * The token is injected via URL search params.
 */
const LoadingWrapper = () => {
  const url = new URL(window.location.href);
  url.searchParams.set('token', 'mock-verify-token');
  window.history.replaceState({}, '', url.toString());
  return <VerifyEmailRoute />;
};

export const Loading: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/verify-email': () => new Promise(() => {}), // never resolves
    }),
  ],
  render: () => <LoadingWrapper />,
};

/**
 * Success state: email has been verified successfully.
 * Shows a green confirmation message and link to sign in.
 */
const SuccessWrapper = () => {
  const url = new URL(window.location.href);
  url.searchParams.set('token', 'mock-verify-token');
  window.history.replaceState({}, '', url.toString());
  return <VerifyEmailRoute />;
};

export const Success: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/verify-email': { message: 'Email verified' },
    }),
  ],
  render: () => <SuccessWrapper />,
};

/**
 * Error state: invalid or expired verification token.
 * Shows a red error message and link back to sign in.
 */
const ErrorWrapper = () => {
  const url = new URL(window.location.href);
  url.searchParams.set('token', 'expired-token');
  window.history.replaceState({}, '', url.toString());
  return <VerifyEmailRoute />;
};

export const Error: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/verify-email': () => {
        throw new window.Error('Verification link has expired. Please request a new one.');
      },
    }),
  ],
  render: () => <ErrorWrapper />,
};
