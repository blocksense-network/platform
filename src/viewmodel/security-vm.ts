/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Security ViewModel
 *
 * Pure utility functions for output encoding and redirect validation.
 * No framework dependencies -- consumed by UI components for safe rendering.
 */

/**
 * Escape HTML entities in untrusted text for safe rendering in HTML context.
 *
 * Prevents XSS by converting characters that have special meaning in HTML
 * into their corresponding entity references.
 */
export function sanitizeDisplayText(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/**
 * Validate that a URL is safe to redirect to.
 *
 * Only allows relative URLs (starting with a single `/`) to prevent
 * open-redirect attacks. Rejects protocol-relative URLs (`//evil.com`),
 * absolute URLs (`https://evil.com`), and other schemes.
 *
 * @returns `true` when the URL is a safe relative path, `false` otherwise.
 */
export function isValidRedirectUrl(url: string): boolean {
  // Must start with "/" but not "//" (protocol-relative)
  if (url.startsWith('/') && !url.startsWith('//')) {
    return true;
  }
  return false;
}
