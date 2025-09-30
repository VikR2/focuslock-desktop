import { useQuery } from '@tanstack/react-query';
import { apiRequest } from '@/lib/queryClient';
import type { AppSummary } from '@shared/schema';

// Check if running in Tauri
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

interface TauriAppInfo {
  name: string;
  path?: string;
  icon?: string;
}

// Convert Tauri app info to AppSummary format
function convertTauriApp(app: TauriAppInfo): AppSummary {
  return {
    appId: app.path || app.name.toLowerCase().replace(/\s+/g, '-'),
    displayName: app.name,
    exeOrTarget: app.path,
    iconHint: app.name.toLowerCase().split(/[\s.]+/)[0], // First word as icon hint
  };
}

// Search for apps with a query string
export function useSearchApps(query: string) {
  return useQuery<AppSummary[]>({
    queryKey: ['/api/apps/search', query, isTauri ? 'tauri' : 'web'],
    queryFn: async () => {
      if (!query || query.length < 2) {
        return [];
      }

      // Use Tauri native API when running in desktop mode
      if (isTauri) {
        try {
          // Try window API first, then dynamic import
          const tauriInvoke = (window as any).__TAURI__?.core?.invoke;
          const invoke = tauriInvoke || (await import(/* @vite-ignore */ '@tauri-apps/api/core')).invoke;
          
          // Get both installed apps and running processes
          const [installedApps, runningProcesses] = await Promise.all([
            invoke('get_installed_apps').then((r: any) => r as TauriAppInfo[]).catch(() => [] as TauriAppInfo[]),
            invoke('get_running_processes').then((r: any) => r as TauriAppInfo[]).catch(() => [] as TauriAppInfo[]),
          ]);
          
          // Combine and deduplicate
          const allApps = new Map<string, TauriAppInfo>();
          
          [...installedApps, ...runningProcesses].forEach(app => {
            const key = app.name.toLowerCase();
            if (!allApps.has(key)) {
              allApps.set(key, app);
            }
          });
          
          // Filter by search query
          const searchLower = query.toLowerCase();
          const filtered = Array.from(allApps.values())
            .filter(app => app.name.toLowerCase().includes(searchLower))
            .map(convertTauriApp);
          
          return filtered;
        } catch (error) {
          console.error('Tauri invoke error:', error);
          // Fall back to web API if Tauri fails
        }
      }

      // Fallback to web API
      const response = await apiRequest('GET', `/api/apps/search?q=${encodeURIComponent(query)}`);
      if (!response.ok) {
        throw new Error(`Failed to search apps: ${response.statusText}`);
      }
      return response.json();
    },
    enabled: query.length >= 2, // Only run query if we have at least 2 characters
    staleTime: 5 * 60 * 1000, // 5 minutes - search results can be cached for a while
  });
}

// Get all available apps (for potential future use)
export function useAllApps() {
  return useQuery<AppSummary[]>({
    queryKey: ['/api/apps'],
    queryFn: async () => {
      const response = await apiRequest('GET', '/api/apps');
      if (!response.ok) {
        throw new Error(`Failed to get apps: ${response.statusText}`);
      }
      return response.json();
    },
    staleTime: 10 * 60 * 1000, // 10 minutes - all apps list doesn't change often
  });
}