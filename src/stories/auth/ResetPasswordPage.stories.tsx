/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Password Reset Page.
 *
 * Two modes:
 * - Request mode (no token): email input form
 * - Confirm mode (with token): new password form
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import ResetPasswordRoute from '~/routes/auth/reset-password';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Auth/ResetPasswordPage',
  component: ResetPasswordRoute,
  decorators: [
    withPageLayout,
    withMockApi({
      '/api/v1/auth/reset-password': {
        message: 'If an account exists with that email, a reset link has been sent.',
      },
      '/api/v1/auth/confirm-reset': { message: 'Password has been reset successfully.' },
    }),
  ],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof ResetPasswordRoute>;

type Story = StoryObj<typeof ResetPasswordRoute>;

/**
 * Request mode: email input form to request a password reset link.
 * This is the default view when no token is present in the URL.
 */
export const RequestMode: Story = {};

/**
 * Error state: simulates a server-side failure on the request.
 */
export const RequestWithError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/reset-password': () => {
        throw new Error('Request failed. Please try again.');
      },
    }),
  ],
};

/**
 * Request success: "Reset link sent" confirmation message.
 * Fills in a valid email and submits the form to show the success message.
 */
export const RequestSuccess: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const emailInput = document.querySelector('#email') as HTMLInputElement | null;
        if (emailInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            emailInput,
            'user@example.com',
          );
          emailInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          const btn = document.querySelector('button[type="submit"]') as HTMLButtonElement | null;
          btn?.click();
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Confirm mode: new password form shown when a token is present in the URL.
 * Uses a wrapper that injects the ?token= search param before render.
 */
const ConfirmModeWrapper = () => {
  // Simulate token in URL by manipulating search params
  const url = new URL(window.location.href);
  url.searchParams.set('token', 'mock-reset-token-abc123');
  window.history.replaceState({}, '', url.toString());
  return <ResetPasswordRoute />;
};

export const ConfirmMode: Story = {
  render: () => <ConfirmModeWrapper />,
};

/**
 * Confirm mode with error: simulates a failure when resetting password
 * with an expired or invalid token.
 */
export const ConfirmWithError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/confirm-reset': () => {
        throw new Error('Reset link has expired. Please request a new one.');
      },
    }),
  ],
  render: () => <ConfirmModeWrapper />,
};

/**
 * Confirm mode success: password has been successfully changed.
 * Fills in matching passwords and submits the confirm form.
 */
const ConfirmSuccessWrapper = () => {
  const url = new URL(window.location.href);
  url.searchParams.set('token', 'mock-reset-token-abc123');
  window.history.replaceState({}, '', url.toString());

  setTimeout(() => {
    const newPw = document.querySelector('#newPassword') as HTMLInputElement | null;
    const confirmPw = document.querySelector('#confirmPassword') as HTMLInputElement | null;
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
    if (newPw) {
      setter?.call(newPw, 'NewStr0ng1');
      newPw.dispatchEvent(new Event('input', { bubbles: true }));
    }
    if (confirmPw) {
      setter?.call(confirmPw, 'NewStr0ng1');
      confirmPw.dispatchEvent(new Event('input', { bubbles: true }));
    }
    setTimeout(() => {
      const btn = document.querySelector('button[type="submit"]') as HTMLButtonElement | null;
      btn?.click();
    }, 50);
  }, 200);

  return <ResetPasswordRoute />;
};

export const ConfirmSuccess: Story = {
  render: () => <ConfirmSuccessWrapper />,
};
