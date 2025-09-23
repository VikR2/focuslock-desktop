import { useQuery } from '@tanstack/react-query';
import { apiRequest } from '@/lib/queryClient';
import type { AppSummary } from '@shared/schema';

// Search for apps with a query string
export function useSearchApps(query: string) {
  return useQuery<AppSummary[]>({
    queryKey: ['/api/apps/search', query],
    queryFn: async () => {
      if (!query || query.length < 2) {
        return [];
      }

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