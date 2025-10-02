import { useQuery, useMutation } from "@tanstack/react-query";
import { apiRequest, queryClient } from "@/lib/queryClient";
import type { Session, InsertSession } from "@shared/schema";

export function useCurrentSession() {
  return useQuery<Session | null>({
    queryKey: ["/api/sessions/current"],
    queryFn: async () => {
      // Get all sessions and find the running one
      const response = await apiRequest("GET", "/api/sessions");
      const sessions: Session[] = await response.json();
      
      // Find the most recent running or scheduled session
      const currentSession = sessions.find(
        (s) => s.status === "running" || s.status === "scheduled"
      );
      
      return currentSession || null;
    },
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
      queryClient.invalidateQueries({ queryKey: ["/api/sessions"] });
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
      queryClient.invalidateQueries({ queryKey: ["/api/sessions"] });
    },
  });
}
