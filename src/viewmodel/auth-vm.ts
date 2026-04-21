/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Auth ViewModel
 *
 * Pure validation logic for authentication forms. No framework dependencies --
 * consumed by SolidJS route components via simple function calls.
 */

// =============================================================================
// Types
// =============================================================================

/** Lifecycle state of an auth form submission. */
export type AuthFormState = 'idle' | 'submitting' | 'success' | 'error';

/** Shape of the sign-up form data. */
export interface SignUpFormData {
  email: string;
  name: string;
  password: string;
  confirmPassword: string;
  acceptTerms: boolean;
}

// =============================================================================
// Validation helpers
// =============================================================================

/**
 * Validates an email address.
 * @returns Error message string, or null when valid.
 */
export function validateEmail(email: string): string | null {
  if (!email.trim()) {
    return 'Email is required';
  }
  // Simple but practical regex: requires local@domain.tld
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(email)) {
    return 'Please enter a valid email address';
  }
  return null;
}

/**
 * Validates a password against minimum complexity requirements.
 * Rules: min 8 chars, at least 1 uppercase, 1 lowercase, 1 digit.
 * @returns Error message string, or null when valid.
 */
export function validatePassword(password: string): string | null {
  if (!password) {
    return 'Password is required';
  }
  if (password.length < 8) {
    return 'Password must be at least 8 characters';
  }
  if (!/[A-Z]/.test(password)) {
    return 'Password must contain at least one uppercase letter';
  }
  if (!/[a-z]/.test(password)) {
    return 'Password must contain at least one lowercase letter';
  }
  if (!/[0-9]/.test(password)) {
    return 'Password must contain at least one digit';
  }
  return null;
}

/**
 * Validates the full sign-up form.
 * @returns A record of field name to error message. Empty record means valid.
 */
export function validateSignUpForm(data: SignUpFormData): Record<string, string> {
  const errors: Record<string, string> = {};

  const nameError = !data.name.trim() ? 'Name is required' : null;
  if (nameError) errors['name'] = nameError;

  const emailError = validateEmail(data.email);
  if (emailError) errors['email'] = emailError;

  const passwordError = validatePassword(data.password);
  if (passwordError) errors['password'] = passwordError;

  if (data.password !== data.confirmPassword) {
    errors['confirmPassword'] = 'Passwords do not match';
  }

  if (!data.acceptTerms) {
    errors['acceptTerms'] = 'You must accept the terms and conditions';
  }

  return errors;
}

/**
 * Validates the sign-in form.
 * @returns A record of field name to error message. Empty record means valid.
 */
export function validateSignInForm(data: {
  email: string;
  password: string;
}): Record<string, string> {
  const errors: Record<string, string> = {};

  const emailError = validateEmail(data.email);
  if (emailError) errors['email'] = emailError;

  if (!data.password) {
    errors['password'] = 'Password is required';
  }

  return errors;
}

/**
 * Estimates password strength based on length and character variety.
 */
export function getPasswordStrength(password: string): 'weak' | 'fair' | 'strong' {
  if (!password || password.length < 8) return 'weak';

  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[a-z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^A-Za-z0-9]/.test(password)) score++;

  if (score >= 5) return 'strong';
  if (score >= 3) return 'fair';
  return 'weak';
}
