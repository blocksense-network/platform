/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Team ViewModel
 *
 * Pure logic for team management UI: role labels, permission checks,
 * seat counting, and invite validation. No framework dependencies --
 * consumed by SolidJS route components via simple function calls.
 */

// =============================================================================
// Types
// =============================================================================

/** Active tab in the team management page. */
export type TeamTab = 'members' | 'invitations';

/** Organization member roles. */
export type MemberRole = 'admin' | 'operator' | 'viewer';

// =============================================================================
// Role helpers
// =============================================================================

/**
 * Returns a human-readable label for a role.
 */
export function getRoleLabel(role: MemberRole): string {
  switch (role) {
    case 'admin':
      return 'Admin';
    case 'operator':
      return 'Operator';
    case 'viewer':
      return 'Viewer';
  }
}

/**
 * Returns a Tailwind color class name portion for role badges.
 */
export function getRoleBadgeColor(role: MemberRole): string {
  switch (role) {
    case 'admin':
      return 'cyan';
    case 'operator':
      return 'yellow';
    case 'viewer':
      return 'gray';
  }
}

// =============================================================================
// Permission checks
// =============================================================================

/**
 * Whether the current user can change other members' roles.
 * Only admins can change roles.
 */
export function canChangeRole(currentUserRole: MemberRole): boolean {
  return currentUserRole === 'admin';
}

/**
 * Whether the current user can remove a target member.
 * Only admins can remove, and they cannot remove themselves.
 */
export function canRemoveMember(
  currentUserRole: MemberRole,
  targetUserId: string,
  currentUserId: string,
): boolean {
  return currentUserRole === 'admin' && targetUserId !== currentUserId;
}

/**
 * Whether the current user can send invitations.
 * Only admins can invite.
 */
export function canInvite(currentUserRole: MemberRole): boolean {
  return currentUserRole === 'admin';
}

// =============================================================================
// Seat counting
// =============================================================================

/**
 * Formats the member count for display.
 * @example "3 / 10 seats" or "3 seats (unlimited)"
 */
export function formatMemberCount(count: number, limit: number | null): string {
  if (limit === null) {
    return `${count} seats (unlimited)`;
  }
  return `${count} / ${limit} seats`;
}

/**
 * Whether the organization is at or over its seat limit.
 * Takes pending invitations into account.
 * Always returns false when limit is null (unlimited).
 */
export function isAtSeatLimit(
  memberCount: number,
  pendingCount: number,
  limit: number | null,
): boolean {
  if (limit === null) {
    return false;
  }
  return memberCount + pendingCount >= limit;
}

// =============================================================================
// Invite validation
// =============================================================================

/**
 * Validates an email address for invitation.
 * @returns Error message string, or null when valid.
 */
export function validateInviteEmail(email: string): string | null {
  if (!email.trim()) {
    return 'Email is required';
  }
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    return 'Please enter a valid email address';
  }
  return null;
}
