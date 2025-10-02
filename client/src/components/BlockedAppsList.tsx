import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Shield, ShieldAlert } from "lucide-react";
import { useBlockRules } from "@/hooks/useBlockRules";
import { useFavorites } from "@/hooks/useFavorites";
import type { BlockRule, Favorite } from "@shared/schema";

export default function BlockedAppsList() {
  const { data: blockRules = [] } = useBlockRules();
  const { data: favorites = [] } = useFavorites();
  
  // Get display names for blocked apps from favorites
  const blockedApps = (blockRules as BlockRule[]).map((rule) => {
    const favorite = (favorites as Favorite[]).find((fav) => fav.appId === rule.appId);
    return {
      ...rule,
      displayName: favorite?.displayName || rule.appId,
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
            <div
              key={app.id}
              className="flex items-center justify-between p-2 rounded-md border border-border hover-elevate"
              data-testid={`blocked-app-${app.appId}`}
            >
              <div className="flex items-center gap-2 flex-1 min-w-0">
                {app.mode === 'hard' ? (
                  <ShieldAlert className="w-4 h-4 text-destructive flex-shrink-0" />
                ) : (
                  <Shield className="w-4 h-4 text-muted-foreground flex-shrink-0" />
                )}
                <span className="text-sm truncate" data-testid={`text-app-name-${app.appId}`}>
                  {app.displayName}
                </span>
              </div>
              <Badge
                variant={app.mode === 'hard' ? 'destructive' : 'secondary'}
                className="text-xs flex-shrink-0"
                data-testid={`badge-mode-${app.appId}`}
              >
                {app.mode === 'hard' ? 'Hard' : 'Soft'}
              </Badge>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
