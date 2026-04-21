/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Executors ViewModel (M11)
 *
 * Pure logic for the managed executors UI: status display, uptime and cost
 * calculations, machine class formatting, and form validation. No framework
 * dependencies -- consumed by SolidJS route components via simple function calls.
 */

import type { MachineClass } from '@agent-harbor/rest-client';

// =============================================================================
// Types
// =============================================================================

/** Managed executor status values. */
export type ExecutorStatus = 'provisioning' | 'running' | 'stopped' | 'terminated';

/** Tab selection for the executors page. */
export type ExecutorTab = 'active' | 'history';

// =============================================================================
// Status display
// =============================================================================

/**
 * Returns a Tailwind color class name for an executor status badge.
 */
export function getStatusColor(status: ExecutorStatus): string {
  switch (status) {
    case 'provisioning':
      return 'yellow';
    case 'running':
      return 'green';
    case 'stopped':
      return 'gray';
    case 'terminated':
      return 'red';
    default:
      return 'gray';
  }
}

/**
 * Returns a human-readable label for an executor status.
 */
export function getStatusLabel(status: ExecutorStatus): string {
  switch (status) {
    case 'provisioning':
      return 'Provisioning';
    case 'running':
      return 'Running';
    case 'stopped':
      return 'Stopped';
    case 'terminated':
      return 'Terminated';
    default:
      return status;
  }
}

// =============================================================================
// Action guards
// =============================================================================

/**
 * Whether the executor can be stopped (only when running).
 */
export function canStop(status: ExecutorStatus): boolean {
  return status === 'running';
}

/**
 * Whether the executor can be started (only when stopped).
 */
export function canStart(status: ExecutorStatus): boolean {
  return status === 'stopped';
}

/**
 * Whether the executor can be terminated (when running or stopped).
 */
export function canTerminate(status: ExecutorStatus): boolean {
  return status === 'running' || status === 'stopped';
}

// =============================================================================
// Uptime & cost calculations
// =============================================================================

/**
 * Calculates human-readable uptime from provisioned/terminated timestamps.
 *
 * @returns e.g. "3h 25m" or "\u2014" if not provisioned
 */
export function calculateUptime(provisionedAt: string | null, terminatedAt: string | null): string {
  if (!provisionedAt) return '\u2014';

  const start = new Date(provisionedAt).getTime();
  const end = terminatedAt ? new Date(terminatedAt).getTime() : Date.now();
  const diffMs = Math.max(0, end - start);

  const totalMinutes = Math.floor(diffMs / 60000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours === 0) return `${minutes}m`;
  return `${hours}h ${minutes}m`;
}

/**
 * Calculates the running cost in cents based on uptime and hourly rate.
 */
export function calculateRunningCost(
  provisionedAt: string | null,
  hourlyRateCents: number,
  terminatedAt: string | null,
): number {
  if (!provisionedAt) return 0;

  const start = new Date(provisionedAt).getTime();
  const end = terminatedAt ? new Date(terminatedAt).getTime() : Date.now();
  const diffMs = Math.max(0, end - start);

  const hours = diffMs / 3600000;
  return Math.round(hours * hourlyRateCents);
}

// =============================================================================
// Machine class formatting
// =============================================================================

/**
 * Formats a machine class specification for display.
 *
 * @example formatMachineClassSpec(mc) => "4 vCPU, 16 GB RAM, 100 GB SSD"
 */
export function formatMachineClassSpec(mc: MachineClass): string {
  return `${mc.vcpus} vCPU, ${mc.ramGb} GB RAM, ${mc.storageGb} GB ${mc.storageType}`;
}

/**
 * Formats an hourly rate in cents for display.
 *
 * @example formatHourlyRate(50) => "$0.50/hr"
 */
export function formatHourlyRate(cents: number): string {
  const dollars = (cents / 100).toFixed(2);
  return `$${dollars}/hr`;
}

// =============================================================================
// Form validation
// =============================================================================

/**
 * Validates the provision executor form.
 *
 * @returns Record of field names to error messages (empty if valid)
 */
export function validateProvisionForm(data: {
  machineClass: string;
  os: string;
  region: string;
}): Record<string, string> {
  const errors: Record<string, string> = {};

  if (!data.machineClass) {
    errors['machineClass'] = 'Machine class is required';
  }
  if (!data.os) {
    errors['os'] = 'Operating system is required';
  }
  if (!data.region) {
    errors['region'] = 'Region is required';
  }

  return errors;
}
