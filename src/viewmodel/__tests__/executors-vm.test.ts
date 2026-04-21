/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Executors ViewModel Tests (M11)
 */

import { describe, it, expect, vi } from 'vitest';
import {
  getStatusColor,
  getStatusLabel,
  canStop,
  canStart,
  canTerminate,
  calculateUptime,
  calculateRunningCost,
  formatMachineClassSpec,
  formatHourlyRate,
  validateProvisionForm,
} from '../executors-vm';
import type { MachineClass } from '@agent-harbor/rest-client';

describe('Executors ViewModel', () => {
  // ===========================================================================
  // getStatusColor
  // ===========================================================================
  describe('getStatusColor', () => {
    it('returns yellow for provisioning', () => {
      expect(getStatusColor('provisioning')).toBe('yellow');
    });

    it('returns green for running', () => {
      expect(getStatusColor('running')).toBe('green');
    });

    it('returns gray for stopped', () => {
      expect(getStatusColor('stopped')).toBe('gray');
    });

    it('returns red for terminated', () => {
      expect(getStatusColor('terminated')).toBe('red');
    });
  });

  // ===========================================================================
  // getStatusLabel
  // ===========================================================================
  describe('getStatusLabel', () => {
    it('returns Provisioning for provisioning', () => {
      expect(getStatusLabel('provisioning')).toBe('Provisioning');
    });

    it('returns Running for running', () => {
      expect(getStatusLabel('running')).toBe('Running');
    });

    it('returns Stopped for stopped', () => {
      expect(getStatusLabel('stopped')).toBe('Stopped');
    });

    it('returns Terminated for terminated', () => {
      expect(getStatusLabel('terminated')).toBe('Terminated');
    });
  });

  // ===========================================================================
  // canStop / canStart / canTerminate
  // ===========================================================================
  describe('canStop', () => {
    it('returns true for running', () => {
      expect(canStop('running')).toBe(true);
    });

    it('returns false for provisioning', () => {
      expect(canStop('provisioning')).toBe(false);
    });

    it('returns false for stopped', () => {
      expect(canStop('stopped')).toBe(false);
    });

    it('returns false for terminated', () => {
      expect(canStop('terminated')).toBe(false);
    });
  });

  describe('canStart', () => {
    it('returns true for stopped', () => {
      expect(canStart('stopped')).toBe(true);
    });

    it('returns false for running', () => {
      expect(canStart('running')).toBe(false);
    });

    it('returns false for provisioning', () => {
      expect(canStart('provisioning')).toBe(false);
    });

    it('returns false for terminated', () => {
      expect(canStart('terminated')).toBe(false);
    });
  });

  describe('canTerminate', () => {
    it('returns true for running', () => {
      expect(canTerminate('running')).toBe(true);
    });

    it('returns true for stopped', () => {
      expect(canTerminate('stopped')).toBe(true);
    });

    it('returns false for provisioning', () => {
      expect(canTerminate('provisioning')).toBe(false);
    });

    it('returns false for terminated', () => {
      expect(canTerminate('terminated')).toBe(false);
    });
  });

  // ===========================================================================
  // calculateUptime
  // ===========================================================================
  describe('calculateUptime', () => {
    it('returns dash when not provisioned', () => {
      expect(calculateUptime(null, null)).toBe('\u2014');
    });

    it('calculates uptime from provisioned to terminated', () => {
      const provisioned = '2026-04-16T10:00:00Z';
      const terminated = '2026-04-16T13:25:00Z';
      expect(calculateUptime(provisioned, terminated)).toBe('3h 25m');
    });

    it('calculates uptime with only minutes', () => {
      const provisioned = '2026-04-16T10:00:00Z';
      const terminated = '2026-04-16T10:45:00Z';
      expect(calculateUptime(provisioned, terminated)).toBe('45m');
    });

    it('calculates uptime for still-running executor', () => {
      // Mock Date.now to a fixed time
      const now = new Date('2026-04-16T12:30:00Z').getTime();
      vi.spyOn(Date, 'now').mockReturnValue(now);

      const provisioned = '2026-04-16T10:00:00Z';
      expect(calculateUptime(provisioned, null)).toBe('2h 30m');

      vi.restoreAllMocks();
    });

    it('returns 0m for zero duration', () => {
      const ts = '2026-04-16T10:00:00Z';
      expect(calculateUptime(ts, ts)).toBe('0m');
    });
  });

  // ===========================================================================
  // calculateRunningCost
  // ===========================================================================
  describe('calculateRunningCost', () => {
    it('returns 0 when not provisioned', () => {
      expect(calculateRunningCost(null, 100, null)).toBe(0);
    });

    it('calculates cost from provisioned to terminated', () => {
      const provisioned = '2026-04-16T10:00:00Z';
      const terminated = '2026-04-16T12:00:00Z'; // 2 hours
      // 2 hours * 100 cents/hr = 200 cents
      expect(calculateRunningCost(provisioned, 100, terminated)).toBe(200);
    });

    it('calculates cost for still-running executor', () => {
      const now = new Date('2026-04-16T11:00:00Z').getTime();
      vi.spyOn(Date, 'now').mockReturnValue(now);

      const provisioned = '2026-04-16T10:00:00Z';
      // 1 hour * 50 cents/hr = 50 cents
      expect(calculateRunningCost(provisioned, 50, null)).toBe(50);

      vi.restoreAllMocks();
    });

    it('rounds to nearest cent', () => {
      const provisioned = '2026-04-16T10:00:00Z';
      const terminated = '2026-04-16T10:30:00Z'; // 0.5 hours
      // 0.5 hours * 75 cents/hr = 37.5, rounded to 38
      expect(calculateRunningCost(provisioned, 75, terminated)).toBe(38);
    });
  });

  // ===========================================================================
  // formatMachineClassSpec
  // ===========================================================================
  describe('formatMachineClassSpec', () => {
    it('formats machine class specification', () => {
      const mc: MachineClass = {
        id: 'mc-1',
        name: 'Standard',
        vcpus: 4,
        ramGb: 16,
        storageGb: 100,
        storageType: 'SSD',
        hourlyRateCents: 50,
        availableOs: ['ubuntu-22.04'],
        availableRegions: ['us-east-1'],
      };
      expect(formatMachineClassSpec(mc)).toBe('4 vCPU, 16 GB RAM, 100 GB SSD');
    });

    it('formats with NVMe storage type', () => {
      const mc: MachineClass = {
        id: 'mc-2',
        name: 'Performance',
        vcpus: 8,
        ramGb: 32,
        storageGb: 500,
        storageType: 'NVMe',
        hourlyRateCents: 200,
        availableOs: ['ubuntu-22.04'],
        availableRegions: ['us-east-1'],
      };
      expect(formatMachineClassSpec(mc)).toBe('8 vCPU, 32 GB RAM, 500 GB NVMe');
    });
  });

  // ===========================================================================
  // formatHourlyRate
  // ===========================================================================
  describe('formatHourlyRate', () => {
    it('formats 50 cents as $0.50/hr', () => {
      expect(formatHourlyRate(50)).toBe('$0.50/hr');
    });

    it('formats 200 cents as $2.00/hr', () => {
      expect(formatHourlyRate(200)).toBe('$2.00/hr');
    });

    it('formats 0 cents as $0.00/hr', () => {
      expect(formatHourlyRate(0)).toBe('$0.00/hr');
    });

    it('formats 1050 cents as $10.50/hr', () => {
      expect(formatHourlyRate(1050)).toBe('$10.50/hr');
    });
  });

  // ===========================================================================
  // validateProvisionForm
  // ===========================================================================
  describe('validateProvisionForm', () => {
    it('returns empty errors for valid form', () => {
      const errors = validateProvisionForm({
        machineClass: 'mc-1',
        os: 'ubuntu-22.04',
        region: 'us-east-1',
      });
      expect(Object.keys(errors)).toHaveLength(0);
    });

    it('returns error for missing machine class', () => {
      const errors = validateProvisionForm({
        machineClass: '',
        os: 'ubuntu-22.04',
        region: 'us-east-1',
      });
      expect(errors.machineClass).toBe('Machine class is required');
    });

    it('returns error for missing OS', () => {
      const errors = validateProvisionForm({
        machineClass: 'mc-1',
        os: '',
        region: 'us-east-1',
      });
      expect(errors.os).toBe('Operating system is required');
    });

    it('returns error for missing region', () => {
      const errors = validateProvisionForm({
        machineClass: 'mc-1',
        os: 'ubuntu-22.04',
        region: '',
      });
      expect(errors.region).toBe('Region is required');
    });

    it('returns multiple errors for multiple missing fields', () => {
      const errors = validateProvisionForm({
        machineClass: '',
        os: '',
        region: '',
      });
      expect(Object.keys(errors)).toHaveLength(3);
      expect(errors.machineClass).toBeDefined();
      expect(errors.os).toBeDefined();
      expect(errors.region).toBeDefined();
    });
  });
});
