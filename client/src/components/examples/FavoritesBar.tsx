import { useState } from "react";
import FavoritesBar from '../FavoritesBar';
import type { BlockMode } from "@shared/schema";

export default function FavoritesBarExample() {
  const [favorites, setFavorites] = useState([
    {
      id: '1',
      appId: 'discord.exe',
      displayName: 'Discord',
      pinnedOrder: 1,
      iconHint: 'discord',
      isBlocked: true,
      blockMode: 'hard' as BlockMode,
    },
    {
      id: '2', 
      appId: 'chrome.exe',
      displayName: 'Google Chrome',
      pinnedOrder: 2,
      iconHint: 'chrome',
      isBlocked: false,
      blockMode: 'soft' as BlockMode,
    },
    {
      id: '3',
      appId: 'steam.exe', 
      displayName: 'Steam',
      pinnedOrder: 3,
      iconHint: 'steam',
      isBlocked: true,
      blockMode: 'soft' as BlockMode,
    },
  ]);

  const handleToggleBlock = (appId: string) => {
    setFavorites(prev => prev.map(fav => 
      fav.appId === appId ? { ...fav, isBlocked: !fav.isBlocked } : fav
    ));
    console.log(`Toggled block for ${appId}`);
  };

  const handleToggleMode = (appId: string, mode: BlockMode) => {
    setFavorites(prev => prev.map(fav => 
      fav.appId === appId ? { ...fav, blockMode: mode } : fav
    ));
    console.log(`Changed block mode for ${appId} to ${mode}`);
  };

  const handleRemoveFavorite = (id: string) => {
    setFavorites(prev => prev.filter(fav => fav.id !== id));
    console.log(`Removed favorite ${id}`);
  };

  const handleAddFavorite = () => {
    console.log('Add favorite clicked');
  };

  return (
    <div className="h-96">
      <FavoritesBar 
        favorites={favorites}
        onToggleBlock={handleToggleBlock}
        onToggleMode={handleToggleMode}
        onRemoveFavorite={handleRemoveFavorite}
        onAddFavorite={handleAddFavorite}
      />
    </div>
  );
}