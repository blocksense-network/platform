/**
 * Copyright 2025 Schelling Point Labs Inc
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * Shared Storybook decorators for platform stories.
 *
 * Provides mock fetch interceptors and PlatformProvider wrappers so that
 * route-level page components can render in Storybook without a real backend.
 */

import { PlatformProvider } from '~/contexts/PlatformContext';
import type { PlatformCapabilities } from '~/contexts/PlatformContext';

/**
 * Default platform capabilities for stories (all features enabled).
 */
const ALL_ENABLED: PlatformCapabilities = {
  mode: 'hosted',
  auth: true,
  billing: true,
  teams: true,
  managedExecutors: true,
};

/**
 * Creates a decorator that intercepts fetch calls and returns mock responses.
 *
 * @param handlers - Map of URL substring patterns to mock response data.
 *   If a handler value is a function, it receives the URL and request init.
 * @param capabilities - Platform capabilities to provide (defaults to all enabled).
 */
export function withMockApi(
  handlers: Record<string, unknown | ((url: string, init?: RequestInit) => unknown)>,
  capabilities: PlatformCapabilities = ALL_ENABLED,
) {
  return (Story: () => any) => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = ((url: string | URL | Request, init?: RequestInit) => {
      const urlStr = typeof url === 'string' ? url : url instanceof URL ? url.href : url.url;

      // Always handle capabilities endpoint
      if (urlStr.includes('/api/v1/platform/capabilities')) {
        return Promise.resolve(
          new Response(JSON.stringify(capabilities), {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
          }),
        );
      }

      // Check custom handlers
      for (const [pattern, handler] of Object.entries(handlers)) {
        if (urlStr.includes(pattern)) {
          try {
            const data = typeof handler === 'function' ? handler(urlStr, init) : handler;
            return Promise.resolve(
              new Response(JSON.stringify(data), {
                status: 200,
                headers: { 'Content-Type': 'application/json' },
              }),
            );
          } catch (err) {
            // Handler threw: return a 400 error response so apiClient treats it as a failure
            const message = err instanceof Error ? err.message : 'Mock error';
            return Promise.resolve(
              new Response(JSON.stringify({ error: message }), {
                status: 400,
                statusText: message,
                headers: { 'Content-Type': 'application/json' },
              }),
            );
          }
        }
      }

      // Fallback: return empty success for any unmatched API calls
      if (urlStr.includes('/api/')) {
        return Promise.resolve(
          new Response(JSON.stringify({}), {
            status: 200,
            headers: { 'Content-Type': 'application/json' },
          }),
        );
      }

      return originalFetch(urlStr, init);
    }) as typeof fetch;

    return (
      <PlatformProvider>
        <Story />
      </PlatformProvider>
    );
  };
}

/**
 * A simple decorator that wraps the story in a PlatformProvider with all
 * capabilities enabled and a catch-all mock fetch for API calls.
 */
export function withPlatformEnabled() {
  return withMockApi({});
}

/**
 * A decorator providing a full-width page layout wrapper.
 */
export function withPageLayout(Story: () => any) {
  return (
    <div class="min-h-screen bg-surface-page text-primary">
      <Story />
    </div>
  );
}
