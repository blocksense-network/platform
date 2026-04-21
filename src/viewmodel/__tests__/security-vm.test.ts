/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Security ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import { sanitizeDisplayText, isValidRedirectUrl } from '../security-vm';

describe('Security ViewModel', () => {
  // ===========================================================================
  // sanitizeDisplayText
  // ===========================================================================
  describe('sanitizeDisplayText', () => {
    it('escapes <script> tags', () => {
      const input = '<script>alert("xss")</script>';
      const result = sanitizeDisplayText(input);
      expect(result).toBe('&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;');
      expect(result).not.toContain('<script>');
    });

    it('escapes ampersands', () => {
      expect(sanitizeDisplayText('A & B')).toBe('A &amp; B');
    });

    it('escapes double quotes', () => {
      expect(sanitizeDisplayText('say "hello"')).toBe('say &quot;hello&quot;');
    });

    it('escapes greater-than signs', () => {
      expect(sanitizeDisplayText('a > b')).toBe('a &gt; b');
    });

    it('escapes less-than signs', () => {
      expect(sanitizeDisplayText('a < b')).toBe('a &lt; b');
    });

    it('handles mixed HTML entities', () => {
      const input = '<img src="x" onerror="alert(1)">&';
      const result = sanitizeDisplayText(input);
      expect(result).toBe('&lt;img src=&quot;x&quot; onerror=&quot;alert(1)&quot;&gt;&amp;');
    });

    it('returns empty string for empty input', () => {
      expect(sanitizeDisplayText('')).toBe('');
    });

    it('leaves safe text untouched', () => {
      expect(sanitizeDisplayText('Hello, World!')).toBe('Hello, World!');
    });
  });

  // ===========================================================================
  // isValidRedirectUrl
  // ===========================================================================
  describe('isValidRedirectUrl', () => {
    it('accepts simple relative paths', () => {
      expect(isValidRedirectUrl('/portal')).toBe(true);
      expect(isValidRedirectUrl('/dashboard/settings')).toBe(true);
      expect(isValidRedirectUrl('/')).toBe(true);
    });

    it('accepts relative paths with query strings', () => {
      expect(isValidRedirectUrl('/search?q=test')).toBe(true);
    });

    it('accepts relative paths with hash fragments', () => {
      expect(isValidRedirectUrl('/page#section')).toBe(true);
    });

    it('rejects protocol-relative URLs (open redirect vector)', () => {
      expect(isValidRedirectUrl('//evil.com')).toBe(false);
      expect(isValidRedirectUrl('//evil.com/path')).toBe(false);
    });

    it('rejects absolute URLs with https', () => {
      expect(isValidRedirectUrl('https://evil.com')).toBe(false);
      expect(isValidRedirectUrl('https://evil.com/path')).toBe(false);
    });

    it('rejects absolute URLs with http', () => {
      expect(isValidRedirectUrl('http://evil.com')).toBe(false);
    });

    it('rejects javascript: scheme', () => {
      expect(isValidRedirectUrl('javascript:alert(1)')).toBe(false);
    });

    it('rejects data: scheme', () => {
      expect(isValidRedirectUrl('data:text/html,<h1>hi</h1>')).toBe(false);
    });

    it('rejects empty string', () => {
      expect(isValidRedirectUrl('')).toBe(false);
    });

    it('rejects relative paths without leading slash', () => {
      expect(isValidRedirectUrl('portal')).toBe(false);
    });
  });
});
