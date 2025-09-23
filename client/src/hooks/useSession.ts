import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest, queryClient } from "@/lib/queryClient";
import type { Session, InsertSession } from "@shared/schema";

export function useCurrentSession() {
  return useQuery<Session | null>({
    queryKey: ["/api/sessions/current"],
  });
}

export function useCreateSession() {
  return useMutation({
    mutationFn: async (sessionData: InsertSession): Promise<Session> => {
      const response = await apiRequest("POST", "/api/sessions", sessionData);
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/sessions/current"] });
    },
  });
}

export function useUpdateSession() {
  return useMutation({
    mutationFn: async ({ id, updates }: { id: string; updates: Partial<Session> }): Promise<Session> => {
      const response = await apiRequest("PATCH", `/api/sessions/${id}`, updates);
      return response.json();
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/sessions/current"] });
    },
  });
}