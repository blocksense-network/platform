/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Usage Dashboard ViewModel Tests
 */

import { describe, it, expect } from 'vitest';
import {
  filterDataByRange,
  calculateAverage,
  calculatePeak,
  formatUsageValue,
  calculateCostAccumulation,
  formatCsvRow,
  generateCsvContent,
} from '../usage-vm';
import type { UsageDataPoint, SeatUsageData, ExecutorUsageData } from '@agent-harbor/rest-client';

// =============================================================================
// Helpers
// =============================================================================

/** Creates a date string N days ago from today. */
function daysAgo(n: number): string {
  const d = new Date();
  d.setDate(d.getDate() - n);
  return d.toISOString().slice(0, 10);
}

// =============================================================================
// filterDataByRange
// =============================================================================
describe('filterDataByRange', () => {
  const data: UsageDataPoint[] = [
    { date: daysAgo(100), value: 1 },
    { date: daysAgo(60), value: 2 },
    { date: daysAgo(20), value: 3 },
    { date: daysAgo(5), value: 4 },
    { date: daysAgo(1), value: 5 },
  ];

  it('filters to last 7 days', () => {
    const result = filterDataByRange(data, '7d');
    expect(result).toHaveLength(2);
    expect(result.map(d => d.value)).toEqual([4, 5]);
  });

  it('filters to last 30 days', () => {
    const result = filterDataByRange(data, '30d');
    expect(result).toHaveLength(3);
    expect(result.map(d => d.value)).toEqual([3, 4, 5]);
  });

  it('filters to last 90 days', () => {
    const result = filterDataByRange(data, '90d');
    expect(result).toHaveLength(4);
    expect(result.map(d => d.value)).toEqual([2, 3, 4, 5]);
  });

  it('returns empty array when no data in range', () => {
    const oldData: UsageDataPoint[] = [{ date: daysAgo(200), value: 1 }];
    expect(filterDataByRange(oldData, '7d')).toHaveLength(0);
  });
});

// =============================================================================
// calculateAverage
// =============================================================================
describe('calculateAverage', () => {
  it('calculates average of data points', () => {
    const data: UsageDataPoint[] = [
      { date: '2026-04-01', value: 10 },
      { date: '2026-04-02', value: 20 },
      { date: '2026-04-03', value: 30 },
    ];
    expect(calculateAverage(data)).toBe(20);
  });

  it('returns 0 for empty array', () => {
    expect(calculateAverage([])).toBe(0);
  });

  it('handles single data point', () => {
    const data: UsageDataPoint[] = [{ date: '2026-04-01', value: 42 }];
    expect(calculateAverage(data)).toBe(42);
  });
});

// =============================================================================
// calculatePeak
// =============================================================================
describe('calculatePeak', () => {
  it('returns maximum value', () => {
    const data: UsageDataPoint[] = [
      { date: '2026-04-01', value: 5 },
      { date: '2026-04-02', value: 15 },
      { date: '2026-04-03', value: 8 },
    ];
    expect(calculatePeak(data)).toBe(15);
  });

  it('returns 0 for empty array', () => {
    expect(calculatePeak([])).toBe(0);
  });

  it('handles all equal values', () => {
    const data: UsageDataPoint[] = [
      { date: '2026-04-01', value: 7 },
      { date: '2026-04-02', value: 7 },
    ];
    expect(calculatePeak(data)).toBe(7);
  });
});

// =============================================================================
// formatUsageValue
// =============================================================================
describe('formatUsageValue', () => {
  it('formats singular seat', () => {
    expect(formatUsageValue(1, 'seats')).toBe('1 seat');
  });

  it('formats plural seats', () => {
    expect(formatUsageValue(5, 'seats')).toBe('5 seats');
  });

  it('formats singular hour', () => {
    expect(formatUsageValue(1, 'hours')).toBe('1 hr');
  });

  it('formats fractional hours', () => {
    expect(formatUsageValue(12.5, 'hours')).toBe('12.5 hrs');
  });

  it('formats zero hours', () => {
    expect(formatUsageValue(0, 'hours')).toBe('0 hrs');
  });

  it('formats unknown metric', () => {
    expect(formatUsageValue(42, 'widgets')).toBe('42 widgets');
  });
});

// =============================================================================
// calculateCostAccumulation
// =============================================================================
describe('calculateCostAccumulation', () => {
  it('produces cumulative totals', () => {
    const executorData: ExecutorUsageData = {
      dataPoints: [
        { date: '2026-04-01', value: 10 },
        { date: '2026-04-02', value: 5 },
        { date: '2026-04-03', value: 8 },
      ],
      byMachineClass: {},
    };

    const result = calculateCostAccumulation(executorData);
    expect(result).toEqual([
      { date: '2026-04-01', value: 10 },
      { date: '2026-04-02', value: 15 },
      { date: '2026-04-03', value: 23 },
    ]);
  });

  it('returns empty array for empty data', () => {
    const executorData: ExecutorUsageData = {
      dataPoints: [],
      byMachineClass: {},
    };
    expect(calculateCostAccumulation(executorData)).toEqual([]);
  });
});

// =============================================================================
// formatCsvRow
// =============================================================================
describe('formatCsvRow', () => {
  it('formats a CSV row', () => {
    expect(formatCsvRow('2026-04-01', 'seats', 5)).toBe('2026-04-01,seats,5');
  });
});

// =============================================================================
// generateCsvContent
// =============================================================================
describe('generateCsvContent', () => {
  it('produces valid CSV with headers and data rows', () => {
    const seatData: SeatUsageData = {
      dataPoints: [
        { date: '2026-04-01', value: 3 },
        { date: '2026-04-02', value: 5 },
      ],
      currentSeats: 5,
      seatLimit: 10,
    };

    const executorData: ExecutorUsageData = {
      dataPoints: [{ date: '2026-04-01', value: 12.5 }],
      byMachineClass: { small: { hours: 12.5, cost: 625 } },
    };

    const csv = generateCsvContent(seatData, executorData);
    const lines = csv.split('\n');

    expect(lines[0]).toBe('date,metric,value');
    expect(lines[1]).toBe('2026-04-01,seats,3');
    expect(lines[2]).toBe('2026-04-02,seats,5');
    expect(lines[3]).toBe('2026-04-01,executor_hours,12.5');
    expect(lines).toHaveLength(4);
  });

  it('produces CSV with only header when no data', () => {
    const seatData: SeatUsageData = {
      dataPoints: [],
      currentSeats: 0,
      seatLimit: 2,
    };
    const executorData: ExecutorUsageData = {
      dataPoints: [],
      byMachineClass: {},
    };

    const csv = generateCsvContent(seatData, executorData);
    expect(csv).toBe('date,metric,value');
  });
});
