import { useQuery, useQueries, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiRequest } from '@/lib/queryClient';
import { useToast } from '@/hooks/use-toast';
import type { Setting, InsertSetting } from '@shared/schema';

// Define the settings structure expected by the UI
export interface AppSettings {
  strictMode: boolean;
  autostart: boolean;
  notificationCadence: string;
  defaultBlockMode: 'hard' | 'soft';
  hotkeysEnabled: boolean;
}

// Default settings values
const DEFAULT_SETTINGS: AppSettings = {
  strictMode: false,
  autostart: true,
  notificationCadence: 'normal',
  defaultBlockMode: 'soft',
  hotkeysEnabled: true,
};

// Setting keys that we care about
const SETTING_KEYS = [
  'strictMode',
  'autostart', 
  'notificationCadence',
  'defaultBlockMode',
  'hotkeysEnabled'
] as const;

// Get a single setting by key
export function useSetting(key: string) {
  return useQuery<Setting | null>({
    queryKey: ['/api/settings', key],
    queryFn: async () => {
      const response = await apiRequest('GET', `/api/settings/${key}`);
      if (!response.ok) {
        if (response.status === 404) {
          return null; // Setting doesn't exist yet
        }
        throw new Error(`Failed to fetch setting: ${response.statusText}`);
      }
      return response.json();
    },
  });
}

// Get all app settings and combine them into a structured object
export function useSettings() {
  const settingQueries = useQueries({
    queries: SETTING_KEYS.map(key => ({
      queryKey: ['/api/settings', key],
      queryFn: async () => {
        const response = await apiRequest('GET', `/api/settings/${key}`);
        if (!response.ok) {
          if (response.status === 404) {
            return null; // Setting doesn't exist yet, use default
          }
          throw new Error(`Failed to fetch setting: ${response.statusText}`);
        }
        return response.json();
      },
    })),
  });

  const isLoading = settingQueries.some(query => query.isLoading);
  const error = settingQueries.find(query => query.error)?.error;

  // Combine individual settings into structured object
  const settings: AppSettings = SETTING_KEYS.reduce((acc, key, index) => {
    const settingValue = settingQueries[index]?.data;
    let parsedValue = DEFAULT_SETTINGS[key];

    if (settingValue?.value) {
      try {
        // Try to parse as JSON, fallback to string value
        parsedValue = JSON.parse(settingValue.value);
      } catch {
        parsedValue = settingValue.value;
      }
    }

    return { ...acc, [key]: parsedValue };
  }, {} as AppSettings);

  return {
    data: settings,
    isLoading,
    error,
  };
}

// Set a single setting
export function useSetSetting() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: async ({ key, value }: { key: string; value: any }): Promise<Setting> => {
      const settingData: InsertSetting = {
        key,
        value: typeof value === 'string' ? value : JSON.stringify(value),
      };

      const response = await apiRequest('POST', '/api/settings', settingData);
      if (!response.ok) {
        throw new Error(`Failed to set setting: ${response.statusText}`);
      }
      return response.json();
    },
    onSuccess: (setting) => {
      // Invalidate all settings queries to trigger refetch
      queryClient.invalidateQueries({ queryKey: ['/api/settings'] });
      toast({
        title: "Setting updated",
        description: `${setting.key} has been updated`,
      });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to update setting",
        description: error.message,
      });
    },
  });
}

// Save multiple settings at once (for the Save button)
export function useSaveSettings() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: async (settings: Partial<AppSettings>): Promise<void> => {
      const promises = Object.entries(settings).map(async ([key, value]) => {
        const settingData: InsertSetting = {
          key,
          value: typeof value === 'string' ? value : JSON.stringify(value),
        };

        const response = await apiRequest('POST', '/api/settings', settingData);
        if (!response.ok) {
          throw new Error(`Failed to set ${key}: ${response.statusText}`);
        }
        return response.json();
      });

      await Promise.all(promises);
    },
    onSuccess: () => {
      // Invalidate all settings queries to trigger refetch
      queryClient.invalidateQueries({ queryKey: ['/api/settings'] });
      toast({
        title: "Settings saved",
        description: "All settings have been saved successfully",
      });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to save settings",
        description: error.message,
      });
    },
  });
}