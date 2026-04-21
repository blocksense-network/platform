/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Auth ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  validateEmail,
  validatePassword,
  validateSignUpForm,
  validateSignInForm,
  getPasswordStrength,
  type SignUpFormData,
} from '../auth-vm';

describe('Auth ViewModel', () => {
  // ===========================================================================
  // Email validation
  // ===========================================================================
  describe('validateEmail', () => {
    it('accepts valid emails', () => {
      expect(validateEmail('user@example.com')).toBeNull();
      expect(validateEmail('a.b@c.co')).toBeNull();
      expect(validateEmail('test+tag@domain.org')).toBeNull();
    });

    it('rejects empty email', () => {
      expect(validateEmail('')).toBe('Email is required');
      expect(validateEmail('   ')).toBe('Email is required');
    });

    it('rejects email without @', () => {
      expect(validateEmail('userexample.com')).toBe('Please enter a valid email address');
    });

    it('rejects email without domain', () => {
      expect(validateEmail('user@')).toBe('Please enter a valid email address');
    });

    it('rejects email without TLD', () => {
      expect(validateEmail('user@domain')).toBe('Please enter a valid email address');
    });
  });

  // ===========================================================================
  // Password validation
  // ===========================================================================
  describe('validatePassword', () => {
    it('accepts valid password', () => {
      expect(validatePassword('Secret1x')).toBeNull();
      expect(validatePassword('MyP@ssw0rd!')).toBeNull();
    });

    it('rejects empty password', () => {
      expect(validatePassword('')).toBe('Password is required');
    });

    it('rejects password shorter than 8 chars', () => {
      expect(validatePassword('Ab1')).toBe('Password must be at least 8 characters');
    });

    it('rejects password without uppercase', () => {
      expect(validatePassword('abcdefg1')).toBe(
        'Password must contain at least one uppercase letter',
      );
    });

    it('rejects password without lowercase', () => {
      expect(validatePassword('ABCDEFG1')).toBe(
        'Password must contain at least one lowercase letter',
      );
    });

    it('rejects password without digit', () => {
      expect(validatePassword('Abcdefgh')).toBe('Password must contain at least one digit');
    });
  });

  // ===========================================================================
  // Sign-up form validation
  // ===========================================================================
  describe('validateSignUpForm', () => {
    const validForm: SignUpFormData = {
      email: 'user@example.com',
      name: 'Test User',
      password: 'Secret1x',
      confirmPassword: 'Secret1x',
      acceptTerms: true,
    };

    it('returns empty errors for valid form', () => {
      expect(validateSignUpForm(validForm)).toEqual({});
    });

    it('returns error when name is missing', () => {
      const errors = validateSignUpForm({ ...validForm, name: '' });
      expect(errors.name).toBeDefined();
    });

    it('returns error when email is missing', () => {
      const errors = validateSignUpForm({ ...validForm, email: '' });
      expect(errors.email).toBeDefined();
    });

    it('returns error when password is missing', () => {
      const errors = validateSignUpForm({ ...validForm, password: '' });
      expect(errors.password).toBeDefined();
    });

    it('returns error when passwords do not match', () => {
      const errors = validateSignUpForm({ ...validForm, confirmPassword: 'Different1' });
      expect(errors.confirmPassword).toBe('Passwords do not match');
    });

    it('returns error when terms not accepted', () => {
      const errors = validateSignUpForm({ ...validForm, acceptTerms: false });
      expect(errors.acceptTerms).toBeDefined();
    });

    it('returns multiple errors at once', () => {
      const errors = validateSignUpForm({
        email: '',
        name: '',
        password: '',
        confirmPassword: '',
        acceptTerms: false,
      });
      expect(Object.keys(errors).length).toBeGreaterThanOrEqual(3);
    });
  });

  // ===========================================================================
  // Sign-in form validation
  // ===========================================================================
  describe('validateSignInForm', () => {
    it('returns empty errors for valid form', () => {
      expect(validateSignInForm({ email: 'user@example.com', password: 'anything' })).toEqual({});
    });

    it('returns error when email is missing', () => {
      const errors = validateSignInForm({ email: '', password: 'anything' });
      expect(errors.email).toBeDefined();
    });

    it('returns error when password is missing', () => {
      const errors = validateSignInForm({ email: 'user@example.com', password: '' });
      expect(errors.password).toBeDefined();
    });
  });

  // ===========================================================================
  // Password strength
  // ===========================================================================
  describe('getPasswordStrength', () => {
    it('rates empty/short passwords as weak', () => {
      expect(getPasswordStrength('')).toBe('weak');
      expect(getPasswordStrength('abc')).toBe('weak');
      expect(getPasswordStrength('1234567')).toBe('weak');
    });

    it('rates medium-complexity passwords as fair', () => {
      // score = length>=8(1) + upper(1) + lower(1) = 3 => fair
      expect(getPasswordStrength('Abcdefgh')).toBe('fair');
      // score = length>=8(1) + lower(1) + digit(1) = 3 => fair
      expect(getPasswordStrength('abcdefg1')).toBe('fair');
    });

    it('rates high-complexity passwords as strong', () => {
      expect(getPasswordStrength('MyP@ssw0rd!!')).toBe('strong');
      expect(getPasswordStrength('Str0ng!Pass#')).toBe('strong');
    });
  });
});
