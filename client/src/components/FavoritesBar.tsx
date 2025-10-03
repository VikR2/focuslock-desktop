import { useState, useEffect } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { 
  MoreVertical, 
  Pin, 
  PinOff, 
  Shield, 
  ShieldOff,
  Plus,
  Monitor,
  Loader2,
  Gamepad2,
  Globe,
  Play,
  Music,
  MessageCircle,
  Users
} from "lucide-react";

// Helper function to get icon component
const getIconComponent = (iconName: string) => {
  const iconMap: Record<string, React.ComponentType<any>> = {
    'gamepad-2': Gamepad2,
    'globe': Globe,
    'play': Play,
    'music': Music,
    'message-circle': MessageCircle,
    'users': Users,
    'monitor': Monitor,
  };
  return iconMap[iconName] || Monitor;
};
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useFavorites, useRemoveFavorite } from "@/hooks/useFavorites";
import { useBlockRules, useAddBlockRule, useRemoveBlockRulesByAppId, useUpdateBlockRule } from "@/hooks/useBlockRules";
import type { Favorite, BlockMode, BlockRule } from "@shared/schema";
import { callTauriCommand } from "@/lib/queryClient";

interface FavoriteApp extends Favorite {
  isBlocked: boolean;
  blockMode: BlockMode;
}

interface FavoritesBarProps {
  onAddFavorite: () => void;
}

export default function FavoritesBar({ 
  onAddFavorite 
}: FavoritesBarProps) {
  const [draggedItem, setDraggedItem] = useState<string | null>(null);
  const [appIcons, setAppIcons] = useState<Record<string, string>>({});
  const [openMenuId, setOpenMenuId] = useState<string | null>(null);
  
  // Fetch data from APIs
  const { data: favorites = [], isLoading: favoritesLoading, error: favoritesError } = useFavorites();
  const { data: blockRules = [], isLoading: rulesLoading, error: rulesError } = useBlockRules();
  
  // Fetch icons for each favorite app
  useEffect(() => {
    const fetchIcons = async () => {
      for (const favorite of favorites as Favorite[]) {
        if (!appIcons[favorite.appId] && favorite.iconHint) {
          try {
            const iconData = await callTauriCommand<string>('get_app_icon', { appPath: favorite.iconHint });
            setAppIcons(prev => ({
              ...prev,
              [favorite.appId]: iconData
            }));
          } catch (error) {
            console.warn(`Failed to load icon for ${favorite.appId}:`, error);
          }
        }
      }
    };
    
    if (favorites.length > 0) {
      fetchIcons();
    }
  }, [favorites]);
  
  // Mutations
  const removeFavoriteMutation = useRemoveFavorite();
  const addBlockRuleMutation = useAddBlockRule();
  const removeBlockRulesByAppIdMutation = useRemoveBlockRulesByAppId();
  const updateBlockRuleMutation = useUpdateBlockRule();
  
  // Combine favorites with block rule status
  const favoriteApps: FavoriteApp[] = (favorites as Favorite[]).map((favorite: Favorite) => {
    const blockRule = (blockRules as BlockRule[]).find((rule: BlockRule) => rule.appId === favorite.appId);
    return {
      ...favorite,
      isBlocked: !!blockRule,
      blockMode: (blockRule?.mode as BlockMode) || 'soft',
    };
  });
  
  const handleToggleBlock = async (appId: string) => {
    const hasBlockRule = (blockRules as BlockRule[]).some((rule: BlockRule) => rule.appId === appId);
    
    if (hasBlockRule) {
      // Remove all block rules for this appId to prevent duplicates
      await removeBlockRulesByAppIdMutation.mutateAsync(appId);
    } else {
      // Check for existing rule before adding to prevent duplicates
      const existingRule = (blockRules as BlockRule[]).find((rule: BlockRule) => rule.appId === appId);
      if (!existingRule) {
        await addBlockRuleMutation.mutateAsync({
          appId,
          matchKind: 'exe',
          mode: 'soft',
        });
      }
    }
  };
  
  const handleToggleMode = async (appId: string, mode: BlockMode) => {
    const blockRule = (blockRules as BlockRule[]).find((rule: BlockRule) => rule.appId === appId);
    if (blockRule) {
      await updateBlockRuleMutation.mutateAsync({
        id: blockRule.id,
        updates: { mode },
      });
    }
  };
  
  const handleRemoveFavorite = async (id: string) => {
    await removeFavoriteMutation.mutateAsync(id);
  };
  
  // Show loading state
  if (favoritesLoading || rulesLoading) {
    return (
      <div className="w-16 bg-sidebar border-r border-sidebar-border flex flex-col py-4">
        <div className="flex-1 flex items-center justify-center">
          <Loader2 className="w-6 h-6 animate-spin text-sidebar-foreground" />
        </div>
      </div>
    );
  }
  
  // Show error state if needed
  if (favoritesError || rulesError) {
    return (
      <div className="w-16 bg-sidebar border-r border-sidebar-border flex flex-col py-4">
        <div className="flex-1 flex items-center justify-center p-2">
          <p className="text-xs text-destructive text-center">Failed to load</p>
        </div>
      </div>
    );
  }

  const getAppIcon = (appId: string) => {
    // Using Lucide icons for proper app representation
    const appLower = appId.toLowerCase();
    
    // Check for common app patterns
    if (appLower.includes('discord')) return 'message-circle';
    if (appLower.includes('chrome') || appLower.includes('firefox') || appLower.includes('edge')) return 'globe';
    if (appLower.includes('steam') || appLower.includes('epic') || appLower.includes('origin')) return 'gamepad-2';
    if (appLower.includes('spotify') || appLower.includes('music')) return 'music';
    if (appLower.includes('slack') || appLower.includes('teams') || appLower.includes('zoom')) return 'users';
    if (appLower.includes('vscode') || appLower.includes('code') || appLower.includes('pycharm')) return 'monitor';
    if (appLower.includes('netflix') || appLower.includes('youtube') || appLower.includes('twitch')) return 'play';
    
    return 'monitor';
  };

  return (
    <div className="w-16 bg-sidebar border-r border-sidebar-border flex flex-col py-4">
      {/* Header */}
      <div className="px-2 mb-4">
        <div className="flex items-center justify-center">
          <Monitor className="w-6 h-6 text-sidebar-foreground" />
        </div>
      </div>

      {/* Favorites List */}
      <div className="flex-1 space-y-2 px-2">
        {favoriteApps.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-32 text-center px-1">
            <Monitor className="w-8 h-8 text-muted-foreground mb-2" />
            <p className="text-xs text-muted-foreground">No favorites yet</p>
            <p className="text-xs text-muted-foreground">Add apps to get started</p>
          </div>
        ) : (
          favoriteApps.map((favorite, index) => (
          <div
            key={favorite.id}
            className="relative group"
            draggable
            onDragStart={() => setDraggedItem(favorite.id)}
            onDragEnd={() => setDraggedItem(null)}
          >
            <div 
              className={`
                relative w-12 h-12 rounded-md border-2 transition-all duration-200
                ${favorite.isBlocked 
                  ? 'border-destructive bg-destructive/10' 
                  : 'border-transparent bg-sidebar-accent hover-elevate'
                }
                ${draggedItem === favorite.id ? 'opacity-50' : ''}
              `}
              onContextMenu={(e) => {
                e.preventDefault();
                setOpenMenuId(favorite.id);
              }}
            >
              {/* App Icon */}
              <div className="w-full h-full flex items-center justify-center">
                {appIcons[favorite.appId] ? (
                  <img 
                    src={appIcons[favorite.appId]} 
                    alt={favorite.displayName} 
                    className="w-8 h-8 object-contain"
                  />
                ) : (
                  (() => {
                    const IconComponent = getIconComponent(getAppIcon(favorite.appId));
                    return <IconComponent className="w-6 h-6 text-sidebar-foreground" />;
                  })()
                )}
              </div>

              {/* Block Status Indicator */}
              {favorite.isBlocked && (
                <div className="absolute -top-1 -right-1">
                  <Badge 
                    variant={favorite.blockMode === 'hard' ? 'destructive' : 'secondary'}
                    className="w-5 h-5 p-0 flex items-center justify-center text-xs"
                  >
                    {favorite.blockMode === 'hard' ? '!' : '~'}
                  </Badge>
                </div>
              )}

              {/* Context Menu */}
              <DropdownMenu 
                open={openMenuId === favorite.id}
                onOpenChange={(open) => setOpenMenuId(open ? favorite.id : null)}
              >
                <DropdownMenuTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="absolute inset-0 w-full h-full opacity-0 group-hover:opacity-100 transition-opacity"
                    data-testid={`button-app-menu-${favorite.appId}`}
                  >
                    <MoreVertical className="w-3 h-3" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start" side="right">
                  <DropdownMenuItem 
                    onClick={() => handleToggleBlock(favorite.appId)}
                    data-testid={`menu-toggle-block-${favorite.appId}`}
                  >
                    {favorite.isBlocked ? (
                      <>
                        <ShieldOff className="w-4 h-4 mr-2" />
                        Unblock App
                      </>
                    ) : (
                      <>
                        <Shield className="w-4 h-4 mr-2" />
                        Block App
                      </>
                    )}
                  </DropdownMenuItem>
                  
                  {favorite.isBlocked && (
                    <DropdownMenuItem 
                      onClick={() => handleToggleMode(
                        favorite.appId, 
                        favorite.blockMode === 'hard' ? 'soft' : 'hard'
                      )}
                      data-testid={`menu-toggle-mode-${favorite.appId}`}
                    >
                      {favorite.blockMode === 'hard' ? (
                        <>
                          <ShieldOff className="w-4 h-4 mr-2" />
                          Soft Block
                        </>
                      ) : (
                        <>
                          <Shield className="w-4 h-4 mr-2" />
                          Hard Block
                        </>
                      )}
                    </DropdownMenuItem>
                  )}
                  
                  <DropdownMenuItem 
                    onClick={() => handleRemoveFavorite(favorite.id)}
                    data-testid={`menu-remove-favorite-${favorite.appId}`}
                    className="text-destructive"
                  >
                    <PinOff className="w-4 h-4 mr-2" />
                    Remove from Favorites
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>

            {/* Tooltip on hover */}
            <div className="absolute left-full ml-2 top-1/2 -translate-y-1/2 bg-popover border border-popover-border px-2 py-1 rounded text-xs text-popover-foreground opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap z-50">
              {favorite.displayName}
              {favorite.isBlocked && (
                <span className="ml-1 text-destructive">
                  ({favorite.blockMode} blocked)
                </span>
              )}
            </div>
          </div>
          ))
        )}
      </div>

      {/* Add Favorite Button */}
      <div className="px-2 mt-4">
        <Button
          size="icon"
          variant="ghost"
          onClick={onAddFavorite}
          data-testid="button-add-favorite"
          className="w-12 h-12 border-2 border-dashed border-muted-foreground/30 hover:border-muted-foreground/50"
        >
          <Plus className="w-4 h-4" />
        </Button>
      </div>
    </div>
  );
}