/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Onboarding ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  generateSlug,
  validateOrgForm,
  validateInviteEntry,
  PLAN_FEATURES,
  type OrgFormData,
  type InviteEntry,
} from '../onboarding-vm';

describe('Onboarding ViewModel', () => {
  // ===========================================================================
  // Slug generation
  // ===========================================================================
  describe('generateSlug', () => {
    it('converts name to lowercase slug with hyphens', () => {
      expect(generateSlug('Acme Corp')).toBe('acme-corp');
    });

    it('strips special characters and periods', () => {
      expect(generateSlug('My App 2.0!')).toBe('my-app-20');
    });

    it('returns empty string for empty input', () => {
      expect(generateSlug('')).toBe('');
    });

    it('handles multiple spaces and hyphens', () => {
      expect(generateSlug('  hello   world  ')).toBe('hello-world');
    });

    it('strips leading and trailing hyphens', () => {
      expect(generateSlug('--test--')).toBe('test');
    });
  });

  // ===========================================================================
  // Org form validation
  // ===========================================================================
  describe('validateOrgForm', () => {
    it('returns empty errors for valid form', () => {
      const data: OrgFormData = { name: 'Acme Corp', slug: 'acme-corp' };
      expect(validateOrgForm(data)).toEqual({});
    });

    it('returns error when name is empty', () => {
      const data: OrgFormData = { name: '', slug: 'acme-corp' };
      const errors = validateOrgForm(data);
      expect(errors['name']).toBeDefined();
    });

    it('returns error when slug is empty', () => {
      const data: OrgFormData = { name: 'Acme', slug: '' };
      const errors = validateOrgForm(data);
      expect(errors['slug']).toBeDefined();
    });

    it('returns error for slug with invalid characters', () => {
      const data: OrgFormData = { name: 'Acme', slug: 'ACME_Corp!' };
      const errors = validateOrgForm(data);
      expect(errors['slug']).toBeDefined();
    });

    it('accepts valid slugs', () => {
      expect(validateOrgForm({ name: 'A', slug: 'ab' })).toEqual({});
      expect(validateOrgForm({ name: 'A', slug: 'my-org-123' })).toEqual({});
    });
  });

  // ===========================================================================
  // Invite entry validation
  // ===========================================================================
  describe('validateInviteEntry', () => {
    it('returns empty errors for valid entry', () => {
      const entry: InviteEntry = { email: 'user@example.com', role: 'admin' };
      expect(validateInviteEntry(entry)).toEqual({});
    });

    it('returns error for invalid email', () => {
      const entry: InviteEntry = { email: 'not-an-email', role: 'admin' };
      const errors = validateInviteEntry(entry);
      expect(errors['email']).toBeDefined();
    });

    it('returns error for empty email', () => {
      const entry: InviteEntry = { email: '', role: 'viewer' };
      const errors = validateInviteEntry(entry);
      expect(errors['email']).toBeDefined();
    });

    it('accepts all valid roles', () => {
      expect(validateInviteEntry({ email: 'a@b.com', role: 'admin' })).toEqual({});
      expect(validateInviteEntry({ email: 'a@b.com', role: 'operator' })).toEqual({});
      expect(validateInviteEntry({ email: 'a@b.com', role: 'viewer' })).toEqual({});
    });
  });

  // ===========================================================================
  // Plan features
  // ===========================================================================
  describe('PLAN_FEATURES', () => {
    it('has all three plans defined', () => {
      expect(PLAN_FEATURES['free']).toBeDefined();
      expect(PLAN_FEATURES['team']).toBeDefined();
      expect(PLAN_FEATURES['enterprise']).toBeDefined();
    });

    it('free plan has correct name and feature count', () => {
      expect(PLAN_FEATURES['free'].name).toBe('Free');
      expect(PLAN_FEATURES['free'].features.length).toBe(3);
    });

    it('team plan has correct name and feature count', () => {
      expect(PLAN_FEATURES['team'].name).toBe('Team');
      expect(PLAN_FEATURES['team'].features.length).toBe(5);
    });

    it('enterprise plan has correct name and feature count', () => {
      expect(PLAN_FEATURES['enterprise'].name).toBe('Enterprise');
      expect(PLAN_FEATURES['enterprise'].features.length).toBe(4);
    });
  });
});
