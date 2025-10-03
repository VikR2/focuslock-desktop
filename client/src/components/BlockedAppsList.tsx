import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { 
  Shield, 
  Monitor,
  Gamepad2,
  Globe,
  Music,
  MessageCircle,
  Users,
  Play,
  Trash2
} from "lucide-react";
import { useBlockRules } from "@/hooks/useBlockRules";
import { useFavorites } from "@/hooks/useFavorites";
import { apiRequest, queryClient, callTauriCommand } from "@/lib/queryClient";
import { useMutation } from "@tanstack/react-query";
import { useToast } from "@/hooks/use-toast";
import type { BlockRule, Favorite } from "@shared/schema";

export default function BlockedAppsList() {
  const { data: blockRules = [] } = useBlockRules();
  const { data: favorites = [] } = useFavorites();
  const { toast } = useToast();
  const [appIcons, setAppIcons] = useState<Record<string, string>>({});
  
  // Fetch icons for each blocked app
  useEffect(() => {
    const fetchIcons = async () => {
      for (const rule of blockRules as BlockRule[]) {
        const favorite = (favorites as Favorite[]).find((fav) => fav.appId === rule.appId);
        if (!appIcons[rule.appId] && favorite?.iconHint) {
          try {
            const iconData = await callTauriCommand<string>('get_app_icon', { appPath: favorite.iconHint });
            setAppIcons(prev => ({
              ...prev,
              [rule.appId]: iconData
            }));
          } catch (error) {
            console.warn(`Failed to load icon for ${rule.appId}:`, error);
          }
        }
      }
    };
    
    if (blockRules.length > 0 && favorites.length > 0) {
      fetchIcons();
    }
  }, [blockRules, favorites]);
  
  const deleteBlockRuleMutation = useMutation({
    mutationFn: async (ruleId: string) => {
      return await apiRequest("DELETE", `/api/block-rules/${ruleId}`);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["/api/block-rules"] });
      toast({
        title: "Block rule removed",
        description: "The app is no longer blocked",
      });
    },
    onError: () => {
      toast({
        title: "Error",
        description: "Failed to remove block rule",
        variant: "destructive",
      });
    },
  });
  
  const getAppIcon = (appId: string) => {
    const appLower = appId.toLowerCase();
    
    if (appLower.includes('discord')) return MessageCircle;
    if (appLower.includes('chrome') || appLower.includes('firefox') || appLower.includes('edge')) return Globe;
    if (appLower.includes('steam') || appLower.includes('epic') || appLower.includes('origin')) return Gamepad2;
    if (appLower.includes('spotify') || appLower.includes('music')) return Music;
    if (appLower.includes('slack') || appLower.includes('teams') || appLower.includes('zoom')) return Users;
    if (appLower.includes('netflix') || appLower.includes('youtube') || appLower.includes('twitch')) return Play;
    
    return Monitor;
  };
  
  // Get display names for blocked apps from favorites
  const blockedApps = (blockRules as BlockRule[]).map((rule) => {
    const favorite = (favorites as Favorite[]).find((fav) => fav.appId === rule.appId);
    return {
      ...rule,
      displayName: favorite?.displayName || rule.appId,
      IconComponent: getAppIcon(rule.appId),
    };
  });
  
  if (blockedApps.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-base">
            <Shield className="w-4 h-4" />
            Blocked Apps
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground text-center py-4">
            No apps blocked yet
          </p>
        </CardContent>
      </Card>
    );
  }
  
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Shield className="w-4 h-4" />
          Blocked Apps ({blockedApps.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-2">
          {blockedApps.map((app) => (
            <ContextMenu key={app.id}>
              <ContextMenuTrigger>
                <div
                  className="flex items-center justify-between p-2 rounded-md border border-border hover-elevate"
                  data-testid={`blocked-app-${app.appId}`}
                >
                  <div className="flex items-center gap-2 flex-1 min-w-0">
                    {appIcons[app.appId] ? (
                      <img 
                        src={appIcons[app.appId]} 
                        alt={app.displayName} 
                        className="w-4 h-4 object-contain flex-shrink-0"
                      />
                    ) : (
                      <app.IconComponent className="w-4 h-4 text-sidebar-foreground flex-shrink-0" />
                    )}
                    <span className="text-sm truncate" data-testid={`text-app-name-${app.appId}`}>
                      {app.displayName}
                    </span>
                  </div>
                </div>
              </ContextMenuTrigger>
              <ContextMenuContent>
                <ContextMenuItem
                  onClick={() => deleteBlockRuleMutation.mutate(app.id)}
                  data-testid={`context-remove-block-${app.appId}`}
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Remove Block
                </ContextMenuItem>
              </ContextMenuContent>
            </ContextMenu>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
