/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Organization Settings ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  validateOrgName,
  validateDeleteConfirmation,
  formatOrgCreatedDate,
} from '../org-settings-vm';

describe('Organization Settings ViewModel', () => {
  // ===========================================================================
  // validateOrgName
  // ===========================================================================
  describe('validateOrgName', () => {
    it('accepts a valid name', () => {
      expect(validateOrgName('Acme Corp')).toBeNull();
    });

    it('accepts a single-character name', () => {
      expect(validateOrgName('A')).toBeNull();
    });

    it('rejects an empty name', () => {
      expect(validateOrgName('')).toBe('Organization name is required');
    });

    it('rejects a whitespace-only name', () => {
      expect(validateOrgName('   ')).toBe('Organization name is required');
    });

    it('rejects a name exceeding 255 characters', () => {
      const longName = 'a'.repeat(256);
      expect(validateOrgName(longName)).toBe('Organization name must be 255 characters or fewer');
    });

    it('accepts a name at exactly 255 characters', () => {
      const maxName = 'a'.repeat(255);
      expect(validateOrgName(maxName)).toBeNull();
    });
  });

  // ===========================================================================
  // validateDeleteConfirmation
  // ===========================================================================
  describe('validateDeleteConfirmation', () => {
    it('returns true for exact match', () => {
      expect(validateDeleteConfirmation('acme-corp', 'acme-corp')).toBe(true);
    });

    it('returns false for wrong slug', () => {
      expect(validateDeleteConfirmation('wrong-slug', 'acme-corp')).toBe(false);
    });

    it('is case-sensitive', () => {
      expect(validateDeleteConfirmation('Acme-Corp', 'acme-corp')).toBe(false);
    });

    it('returns false for empty input', () => {
      expect(validateDeleteConfirmation('', 'acme-corp')).toBe(false);
    });

    it('returns false for partial match', () => {
      expect(validateDeleteConfirmation('acme', 'acme-corp')).toBe(false);
    });
  });

  // ===========================================================================
  // formatOrgCreatedDate
  // ===========================================================================
  describe('formatOrgCreatedDate', () => {
    it('formats an ISO date string', () => {
      const result = formatOrgCreatedDate('2026-04-19T10:00:00Z');
      expect(result).toBe('Created on Apr 19, 2026');
    });

    it('formats another date', () => {
      const result = formatOrgCreatedDate('2025-01-15T00:00:00Z');
      expect(result).toBe('Created on Jan 15, 2025');
    });

    it('handles an invalid date gracefully', () => {
      const result = formatOrgCreatedDate('not-a-date');
      // Should still start with "Created on"
      expect(result).toMatch(/^Created on /);
    });
  });
});
