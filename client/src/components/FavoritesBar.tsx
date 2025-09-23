import { useState } from "react";
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
  Monitor
} from "lucide-react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Favorite, BlockMode } from "@shared/schema";

interface FavoriteApp extends Favorite {
  isBlocked: boolean;
  blockMode: BlockMode;
}

interface FavoritesBarProps {
  favorites: FavoriteApp[];
  onToggleBlock: (appId: string) => void;
  onToggleMode: (appId: string, mode: BlockMode) => void;
  onRemoveFavorite: (id: string) => void;
  onAddFavorite: () => void;
}

export default function FavoritesBar({ 
  favorites, 
  onToggleBlock, 
  onToggleMode, 
  onRemoveFavorite,
  onAddFavorite 
}: FavoritesBarProps) {
  const [draggedItem, setDraggedItem] = useState<string | null>(null);

  const getAppIcon = (appId: string) => {
    // todo: remove mock functionality - In real app, would show actual app icons
    const icons: Record<string, string> = {
      'discord.exe': '🎮',
      'chrome.exe': '🌐',
      'steam.exe': '🎯',
      'spotify.exe': '🎵',
      'slack.exe': '💬',
      'teams.exe': '👥',
    };
    return icons[appId] || '📱';
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
        {favorites.map((favorite, index) => (
          <div
            key={favorite.id}
            className="relative group"
            draggable
            onDragStart={() => setDraggedItem(favorite.id)}
            onDragEnd={() => setDraggedItem(null)}
          >
            <div className={`
              relative w-12 h-12 rounded-md border-2 transition-all duration-200
              ${favorite.isBlocked 
                ? 'border-destructive bg-destructive/10' 
                : 'border-transparent bg-sidebar-accent hover-elevate'
              }
              ${draggedItem === favorite.id ? 'opacity-50' : ''}
            `}>
              {/* App Icon */}
              <div className="w-full h-full flex items-center justify-center text-lg">
                {getAppIcon(favorite.appId)}
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
              <DropdownMenu>
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
                    onClick={() => onToggleBlock(favorite.appId)}
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
                      onClick={() => onToggleMode(
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
                    onClick={() => onRemoveFavorite(favorite.id)}
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
        ))}
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