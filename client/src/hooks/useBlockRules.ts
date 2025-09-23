import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiRequest } from '@/lib/queryClient';
import { useToast } from '@/hooks/use-toast';
import type { BlockRule, InsertBlockRule } from '@shared/schema';

// Get all block rules
export function useBlockRules() {
  return useQuery<BlockRule[]>({
    queryKey: ['/api/block-rules'],
  });
}

// Add a new block rule
export function useAddBlockRule() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async (rule: InsertBlockRule): Promise<BlockRule> => {
      const response = await apiRequest('POST', '/api/block-rules', rule);
      if (!response.ok) {
        throw new Error(`Failed to add block rule: ${response.statusText}`);
      }
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/block-rules'] });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to add block rule",
        description: error.message,
      });
    },
  });
}

// Update a block rule
export function useUpdateBlockRule() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async ({ id, updates }: { id: string; updates: Partial<BlockRule> }): Promise<BlockRule> => {
      const response = await apiRequest('PATCH', `/api/block-rules/${id}`, updates);
      if (!response.ok) {
        throw new Error(`Failed to update block rule: ${response.statusText}`);
      }
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/block-rules'] });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to update block rule",
        description: error.message,
      });
    },
  });
}

// Remove a block rule  
export function useRemoveBlockRule() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async (id: string): Promise<void> => {
      const response = await apiRequest('DELETE', `/api/block-rules/${id}`);
      if (!response.ok) {
        throw new Error(`Failed to remove block rule: ${response.statusText}`);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/block-rules'] });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to remove block rule",
        description: error.message,
      });
    },
  });
}

// Remove all block rules for a specific appId (to prevent duplicates)
export function useRemoveBlockRulesByAppId() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async (appId: string): Promise<void> => {
      // Get current rules to find all matching this appId
      const blockRules = queryClient.getQueryData<BlockRule[]>(['/api/block-rules']) || [];
      const rulesToRemove = blockRules.filter(rule => rule.appId === appId);
      
      // Remove all matching rules
      await Promise.all(
        rulesToRemove.map(rule => 
          apiRequest('DELETE', `/api/block-rules/${rule.id}`)
        )
      );
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/block-rules'] });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to remove block rules",
        description: error.message,
      });
    },
  });
}