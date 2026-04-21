/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Storybook stories for the Onboarding Wizard.
 *
 * Multi-step wizard: Create Organization -> Choose Plan -> Invite Team.
 * The component manages its own step state, so stories start at step 1.
 */

import type { Meta, StoryObj } from 'storybook-solidjs-vite';
import OnboardingRoute from '~/routes/onboarding';
import { withMockApi, withPageLayout } from '../decorators';

export default {
  title: 'Platform/Onboarding/OnboardingWizard',
  component: OnboardingRoute,
  decorators: [
    withPageLayout,
    withMockApi({
      '/api/v1/orgs': { id: 'org-new', name: 'New Org', slug: 'new-org' },
      '/api/v1/orgs/org-new/invitations': { ok: true },
    }),
  ],
  parameters: {
    layout: 'fullscreen',
  },
} satisfies Meta<typeof OnboardingRoute>;

type Story = StoryObj<typeof OnboardingRoute>;

/**
 * Step 1: Create Organization.
 * The initial state with organization name and slug inputs.
 */
export const Step1CreateOrganization: Story = {};

/**
 * Error state: simulates a server error when creating the organization.
 */
export const WithServerError: Story = {
  decorators: [
    withMockApi({
      '/api/v1/orgs': () => {
        throw new Error('Organization slug is already taken');
      },
    }),
  ],
};

/**
 * Step 1 with validation errors: empty org name and slug.
 * Clicks the "Next" button immediately to trigger validation.
 */
export const Step1WithValidation: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        // Find the "Next" button and click it to trigger validation
        const buttons = document.querySelectorAll('button[type="button"]');
        for (const btn of buttons) {
          if (btn.textContent?.trim() === 'Next') {
            (btn as HTMLButtonElement).click();
            break;
          }
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 2: Choose Plan. Shows the plan comparison cards (Free/Team/Enterprise).
 * Fills in valid org data and clicks Next to advance to step 2.
 */
export const Step2ChoosePlan: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'My Organization');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Next') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 2 with Team plan selected. The Team plan card is highlighted.
 */
export const Step2TeamSelected: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'My Organization');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          // Click Next to go to step 2
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Next') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
          // After step 2 renders, click the Team plan card
          setTimeout(() => {
            const planButtons = document.querySelectorAll('button[type="button"]');
            for (const btn of planButtons) {
              if (btn.textContent?.includes('Team')) {
                (btn as HTMLButtonElement).click();
                break;
              }
            }
          }, 100);
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 3: Invite Team. Shows invite email inputs with role selectors
 * and Skip/Finish buttons.
 */
export const Step3InviteTeam: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'My Organization');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          // Click Next to go to step 2
          let nextBtn: HTMLButtonElement | null = null;
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Next') {
              nextBtn = btn as HTMLButtonElement;
              break;
            }
          }
          nextBtn?.click();
          // Click Next again to go to step 3
          setTimeout(() => {
            const buttons2 = document.querySelectorAll('button[type="button"]');
            for (const btn of buttons2) {
              if (btn.textContent?.trim() === 'Next') {
                (btn as HTMLButtonElement).click();
                break;
              }
            }
            // Add invite entries after step 3 renders
            setTimeout(() => {
              const emailInputs = document.querySelectorAll('input[type="email"]');
              if (emailInputs.length > 0) {
                setter?.call(emailInputs[0] as HTMLInputElement, 'alice@example.com');
                (emailInputs[0] as HTMLInputElement).dispatchEvent(
                  new Event('input', { bubbles: true }),
                );
              }
              // Click "+ Add another" to add a second row
              const addBtn = document.querySelector('button.font-mono.text-sm.text-cyan-400');
              if (addBtn) {
                (addBtn as HTMLButtonElement).click();
              }
            }, 100);
          }, 100);
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 1 with org name filled: shows auto-generated slug from org name.
 */
export const Step1WithOrgName: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'Acme Corporation');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 2 with Enterprise plan selected: Enterprise plan card highlighted.
 */
export const Step2EnterpriseSelected: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'Big Enterprise');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Next') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
          setTimeout(() => {
            const planButtons = document.querySelectorAll('button[type="button"]');
            for (const btn of planButtons) {
              if (btn.textContent?.includes('Enterprise')) {
                (btn as HTMLButtonElement).click();
                break;
              }
            }
          }, 100);
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};

/**
 * Step 3 empty: invite step with no email entries yet (skip option visible).
 */
export const Step3Empty: Story = {
  decorators: [
    (Story: () => any) => {
      setTimeout(() => {
        const nameInput = document.querySelector('#orgName') as HTMLInputElement | null;
        const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
        if (nameInput) {
          setter?.call(nameInput, 'My Organization');
          nameInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
        setTimeout(() => {
          // Click Next to go to step 2
          const buttons = document.querySelectorAll('button[type="button"]');
          for (const btn of buttons) {
            if (btn.textContent?.trim() === 'Next') {
              (btn as HTMLButtonElement).click();
              break;
            }
          }
          // Click Next again to go to step 3
          setTimeout(() => {
            const buttons2 = document.querySelectorAll('button[type="button"]');
            for (const btn of buttons2) {
              if (btn.textContent?.trim() === 'Next') {
                (btn as HTMLButtonElement).click();
                break;
              }
            }
          }, 100);
        }, 50);
      }, 100);
      return <Story />;
    },
  ],
};
