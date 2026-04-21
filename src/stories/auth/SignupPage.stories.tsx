/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Sign Up Page.
 *
 * Renders the registration route component with mock API responses.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import SignUpRoute from '~/routes/auth/signup';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Auth/SignupPage',
  component: SignUpRoute,
  decorators: [
    withPageLayout,
    withMockApi({
      '/api/v1/auth/signup': { message: 'Account created! Please check your email to verify.' },
    }),
  ],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof SignUpRoute>;

type Story = StoryObj<typeof SignUpRoute>;

/**
 * Empty form (default idle state).
 * Shows name, email, password, confirm password fields and terms checkbox.
 */
export const Default: Story = {};

/**
 * Error state: simulates a server-side registration failure.
 */
export const WithServerError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/auth/signup': () => {
        throw new Error('An account with this email already exists');
      },
    }),
  ],
};

/**
 * Validation errors: form submitted with all fields empty.
 * Shows error messages for name, email, password, confirm, and terms.
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
 * Password strength indicator showing "weak" for a short password.
 */
export const PasswordStrengthWeak: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const pwInput = document.querySelector('#password') as HTMLInputElement | null;
        if (pwInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            pwInput,
            'abc',
          );
          pwInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Password strength indicator showing "fair" for a medium-complexity password.
 */
export const PasswordStrengthFair: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const pwInput = document.querySelector('#password') as HTMLInputElement | null;
        if (pwInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            pwInput,
            'Abcdefgh',
          );
          pwInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Password strength indicator showing "strong" for a complex password.
 */
export const PasswordStrengthStrong: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const pwInput = document.querySelector('#password') as HTMLInputElement | null;
        if (pwInput) {
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set?.call(
            pwInput,
            'Str0ng!Pass#42',
          );
          pwInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Form partially filled: name and email entered but no password yet.
 * Shows a typical mid-completion state.
 */
export const PartiallyFilled: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        const nameInput = document.querySelector('#name') as HTMLInputElement | null;
        const emailInput = document.querySelector('#email') as HTMLInputElement | null;
        if (nameInput) {
          setter?.call(nameInput, 'Jane Doe');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (emailInput) {
          setter?.call(emailInput, 'jane@example.com');
          emailInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Success state: verification email sent message after successful registration.
 * Uses a mock that returns immediately, and fills in valid form data to trigger submit.
 */
export const SuccessState: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#name') as HTMLInputElement | null;
        const emailInput = document.querySelector('#email') as HTMLInputElement | null;
        const pwInput = document.querySelector('#password') as HTMLInputElement | null;
        const confirmInput = document.querySelector('#confirmPassword') as HTMLInputElement | null;
        const termsInput = document.querySelector('#acceptTerms') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;

        if (nameInput) {
          setter?.call(nameInput, 'Test User');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (emailInput) {
          setter?.call(emailInput, 'test@example.com');
          emailInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (pwInput) {
          setter?.call(pwInput, 'Str0ngPass1');
          pwInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (confirmInput) {
          setter?.call(confirmInput, 'Str0ngPass1');
          confirmInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        if (termsInput && !termsInput.checked) {
          termsInput.click();
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
