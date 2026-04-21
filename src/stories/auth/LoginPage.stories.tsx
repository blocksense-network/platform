/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Login Page.
 *
 * Renders the sign-in route component with mock API responses for
 * various states: idle, validation errors, loading, and server error.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import LoginRoute from '~/routes/auth/login';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Auth/LoginPage',
  component: LoginRoute,
  decorators: [
    withPageLayout,
    withMockApi({
      '/api/v1/auth/signin': {
        accessToken: 'mock-token',
        refreshToken: 'mock-refresh',
        user: { id: 'u-1', email: 'test@example.com' },
      },
    }),
  ],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof LoginRoute>;

type Story = StoryObj<typeof LoginRoute>;

/**
 * Empty form (default idle state).
 * User sees email and password inputs with OAuth placeholder buttons.
 */
export const Default: Story = {};

/**
 * Error state: simulates wrong credentials by returning a failed response.
 * The user must interact with the form to trigger the error display.
 */
export const WithServerError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/signin': (_url: string, _init?: RequestInit) => {
        throw new Error('Invalid email or password');
      },
    }),
  ],
};

/**
 * Validation errors state: form submitted with empty fields.
 * Renders the login form and immediately clicks submit to trigger
 * client-side validation error messages on email and password fields.
 */
export const WithValidationErrors: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const btn = document.querySelector('button[type="submit"]') as HTMLButtonElement | null;
        btn?.click();
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Form with email filled but no password.
 * Shows a partially completed form state.
 */
export const WithEmailOnly: Story = {
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
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Submitting state: form in loading/disabled state.
 * Renders the login form with valid-looking data and a mock endpoint
 * that never resolves, so the button stays in "Signing in..." state.
 */
export const Submitting: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/signin': () => new Promise(() => {}), // never resolves
    }),
    (Story: () => any) => {
      setTimeout(() => {
        const emailInput = document.querySelector('#email') as HTMLInputElement | null;
        const passwordInput = document.querySelector('#password') as HTMLInputElement | null;
        if (emailInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            emailInput,
            'user@example.com',
          );
          emailInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (passwordInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            passwordInput,
            'Password1',
          );
          passwordInput.dispatchEvent(new Event('input', { bubbles: true }));
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
