/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Organization Settings ViewModel
 *
 * Pure logic for the org settings page: name validation, delete confirmation,
 * and date formatting. No framework dependencies -- consumed by SolidJS
 * route components via simple function calls.
 */

// =============================================================================
// Validation
// =============================================================================

/**
 * Validates an organization name.
 * @returns Error message string, or null when valid.
 */
export function validateOrgName(name: string): string | null {
  if (!name.trim()) {
    return 'Organization name is required';
  }
  if (name.length > 255) {
    return 'Organization name must be 255 characters or fewer';
  }
  return null;
}

/**
 * Validates the delete confirmation slug against the actual org slug.
 * Comparison is case-sensitive and must be an exact match.
 */
export function validateDeleteConfirmation(inputSlug: string, actualSlug: string): boolean {
  return inputSlug === actualSlug;
}

// =============================================================================
// Formatting
// =============================================================================

/**
 * Formats an ISO date string into a human-readable "Created on" label.
 * @example formatOrgCreatedDate("2026-04-19T10:00:00Z") => "Created on Apr 19, 2026"
 */
export function formatOrgCreatedDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    const formatted = date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
    return `Created on ${formatted}`;
  } catch {
    return `Created on ${dateStr}`;
  }
}
