/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Team ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  getRoleLabel,
  getRoleBadgeColor,
  canChangeRole,
  canRemoveMember,
  canInvite,
  formatMemberCount,
  isAtSeatLimit,
  validateInviteEmail,
} from '../team-vm';

describe('Team ViewModel', () => {
  // ===========================================================================
  // Role labels
  // ===========================================================================
  describe('getRoleLabel', () => {
    it('returns Admin for admin', () => {
      expect(getRoleLabel('admin')).toBe('Admin');
    });

    it('returns Operator for operator', () => {
      expect(getRoleLabel('operator')).toBe('Operator');
    });

    it('returns Viewer for viewer', () => {
      expect(getRoleLabel('viewer')).toBe('Viewer');
    });
  });

  // ===========================================================================
  // Role badge colors
  // ===========================================================================
  describe('getRoleBadgeColor', () => {
    it('returns cyan for admin', () => {
      expect(getRoleBadgeColor('admin')).toBe('cyan');
    });

    it('returns yellow for operator', () => {
      expect(getRoleBadgeColor('operator')).toBe('yellow');
    });

    it('returns gray for viewer', () => {
      expect(getRoleBadgeColor('viewer')).toBe('gray');
    });
  });

  // ===========================================================================
  // canChangeRole
  // ===========================================================================
  describe('canChangeRole', () => {
    it('returns true for admin', () => {
      expect(canChangeRole('admin')).toBe(true);
    });

    it('returns false for operator', () => {
      expect(canChangeRole('operator')).toBe(false);
    });

    it('returns false for viewer', () => {
      expect(canChangeRole('viewer')).toBe(false);
    });
  });

  // ===========================================================================
  // canRemoveMember
  // ===========================================================================
  describe('canRemoveMember', () => {
    it('returns true when admin removing a different user', () => {
      expect(canRemoveMember('admin', 'user-2', 'user-1')).toBe(true);
    });

    it('returns false when admin removing self', () => {
      expect(canRemoveMember('admin', 'user-1', 'user-1')).toBe(false);
    });

    it('returns false for operator', () => {
      expect(canRemoveMember('operator', 'user-2', 'user-1')).toBe(false);
    });

    it('returns false for viewer', () => {
      expect(canRemoveMember('viewer', 'user-2', 'user-1')).toBe(false);
    });
  });

  // ===========================================================================
  // canInvite
  // ===========================================================================
  describe('canInvite', () => {
    it('returns true for admin', () => {
      expect(canInvite('admin')).toBe(true);
    });

    it('returns false for operator', () => {
      expect(canInvite('operator')).toBe(false);
    });

    it('returns false for viewer', () => {
      expect(canInvite('viewer')).toBe(false);
    });
  });

  // ===========================================================================
  // formatMemberCount
  // ===========================================================================
  describe('formatMemberCount', () => {
    it('formats count with limit', () => {
      expect(formatMemberCount(3, 10)).toBe('3 / 10 seats');
    });

    it('formats count without limit', () => {
      expect(formatMemberCount(3, null)).toBe('3 seats (unlimited)');
    });

    it('formats zero count with limit', () => {
      expect(formatMemberCount(0, 5)).toBe('0 / 5 seats');
    });
  });

  // ===========================================================================
  // isAtSeatLimit
  // ===========================================================================
  describe('isAtSeatLimit', () => {
    it('returns true when at limit', () => {
      expect(isAtSeatLimit(8, 2, 10)).toBe(true);
    });

    it('returns true when over limit', () => {
      expect(isAtSeatLimit(9, 2, 10)).toBe(true);
    });

    it('returns false when under limit', () => {
      expect(isAtSeatLimit(5, 2, 10)).toBe(false);
    });

    it('returns false when unlimited', () => {
      expect(isAtSeatLimit(100, 50, null)).toBe(false);
    });

    it('returns true when exactly at limit with no pending', () => {
      expect(isAtSeatLimit(10, 0, 10)).toBe(true);
    });
  });

  // ===========================================================================
  // validateInviteEmail
  // ===========================================================================
  describe('validateInviteEmail', () => {
    it('accepts valid email', () => {
      expect(validateInviteEmail('user@example.com')).toBeNull();
    });

    it('accepts email with plus tag', () => {
      expect(validateInviteEmail('user+tag@example.com')).toBeNull();
    });

    it('rejects empty email', () => {
      expect(validateInviteEmail('')).toBe('Email is required');
    });

    it('rejects whitespace-only email', () => {
      expect(validateInviteEmail('   ')).toBe('Email is required');
    });

    it('rejects email without @', () => {
      expect(validateInviteEmail('userexample.com')).toBe('Please enter a valid email address');
    });

    it('rejects email without domain', () => {
      expect(validateInviteEmail('user@')).toBe('Please enter a valid email address');
    });

    it('rejects email without TLD', () => {
      expect(validateInviteEmail('user@domain')).toBe('Please enter a valid email address');
    });
  });
});
