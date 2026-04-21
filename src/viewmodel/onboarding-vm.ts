/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Onboarding ViewModel
 *
 * Pure validation and data logic for the onboarding wizard.
 * No framework dependencies -- consumed by SolidJS route components
 * via simple function calls.
 */

// =============================================================================
// Types
// =============================================================================

/** Steps in the onboarding wizard. */
export type OnboardingStep = 'create-org' | 'choose-plan' | 'invite-team';

/** Shape of the organization creation form data. */
export interface OrgFormData {
  name: string;
  slug: string;
}

/** Available plan choices. */
export type PlanChoice = 'free' | 'team' | 'enterprise';

/** A single invitation entry. */
export interface InviteEntry {
  email: string;
  role: 'admin' | 'operator' | 'viewer';
}

// =============================================================================
// Plan features
// =============================================================================

/** Plan feature metadata for display in the plan selection step. */
export const PLAN_FEATURES: Record<
  PlanChoice,
  { name: string; price: string; features: string[]; cta: string }
> = {
  free: {
    name: 'Free',
    price: '$0',
    features: ['Up to 2 team members', 'BYO executors', '100 tasks/month'],
    cta: 'Get Started',
  },
  team: {
    name: 'Team',
    price: '$15/seat/mo',
    features: ['Unlimited members', 'Managed executors', 'Unlimited tasks', 'RBAC', 'Audit logs'],
    cta: 'Start Free Trial',
  },
  enterprise: {
    name: 'Enterprise',
    price: 'Custom',
    features: ['Everything in Team', 'Dedicated executors', 'SSO', 'SLA'],
    cta: 'Contact Sales',
  },
};

// =============================================================================
// Slug generation
// =============================================================================

/**
 * Generates a URL-safe slug from an organization name.
 *
 * Converts to lowercase, replaces spaces and special characters with hyphens,
 * collapses consecutive hyphens, and trims leading/trailing hyphens.
 */
export function generateSlug(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/[\s]+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
}

// =============================================================================
// Validation helpers
// =============================================================================

/**
 * Validates the organization creation form.
 * @returns A record of field name to error message. Empty record means valid.
 */
export function validateOrgForm(data: OrgFormData): Record<string, string> {
  const errors: Record<string, string> = {};

  if (!data.name.trim()) {
    errors['name'] = 'Organization name is required';
  }

  if (!data.slug.trim()) {
    errors['slug'] = 'Slug is required';
  } else if (!/^[a-z0-9][a-z0-9-]*[a-z0-9]$/.test(data.slug) && data.slug.length > 1) {
    errors['slug'] = 'Slug must contain only lowercase letters, numbers, and hyphens';
  } else if (data.slug.length === 1 && !/^[a-z0-9]$/.test(data.slug)) {
    errors['slug'] = 'Slug must contain only lowercase letters, numbers, and hyphens';
  } else if (data.slug.length < 2) {
    errors['slug'] = 'Slug must be at least 2 characters';
  }

  return errors;
}

/**
 * Validates a single invite entry.
 * @returns A record of field name to error message. Empty record means valid.
 */
export function validateInviteEntry(entry: InviteEntry): Record<string, string> {
  const errors: Record<string, string> = {};

  if (!entry.email.trim()) {
    errors['email'] = 'Email is required';
  } else {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(entry.email)) {
      errors['email'] = 'Please enter a valid email address';
    }
  }

  const validRoles = ['admin', 'operator', 'viewer'];
  if (!validRoles.includes(entry.role)) {
    errors['role'] = 'Invalid role';
  }

  return errors;
}
