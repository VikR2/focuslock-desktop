import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiRequest } from '@/lib/queryClient';
import { useToast } from '@/hooks/use-toast';
import type { Favorite, InsertFavorite } from '@shared/schema';

// Get all favorites
export function useFavorites() {
  return useQuery<Favorite[]>({
    queryKey: ['/api/favorites'],
  });
}

// Add a new favorite
export function useAddFavorite() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async (favorite: InsertFavorite): Promise<Favorite> => {
      const response = await apiRequest('POST', '/api/favorites', favorite);
      if (!response.ok) {
        throw new Error(`Failed to add favorite: ${response.statusText}`);
      }
      return response.json();
    },
    onSuccess: (favorite) => {
      queryClient.invalidateQueries({ queryKey: ['/api/favorites'] });
      toast({
        title: "Favorite added",
        description: `${favorite.displayName} has been added to favorites`,
      });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to add favorite",
        description: error.message,
      });
    },
  });
}

// Remove a favorite
export function useRemoveFavorite() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  
  return useMutation({
    mutationFn: async (id: string): Promise<void> => {
      const response = await apiRequest('DELETE', `/api/favorites/${id}`);
      if (!response.ok) {
        throw new Error(`Failed to remove favorite: ${response.statusText}`);
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['/api/favorites'] });
      toast({
        title: "Favorite removed",
        description: "The app has been removed from favorites",
      });
    },
    onError: (error: Error) => {
      toast({
        variant: "destructive",
        title: "Failed to remove favorite",
        description: error.message,
      });
    },
  });
}