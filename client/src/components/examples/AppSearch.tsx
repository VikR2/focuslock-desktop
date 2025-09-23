import { useState } from "react";
import AppSearch from '../AppSearch';
import type { AppSummary, BlockMode } from "@shared/schema";

export default function AppSearchExample() {
  const [blockedApps, setBlockedApps] = useState<string[]>(['discord.exe']);

  const handleAddToBlockList = (app: AppSummary, mode: BlockMode) => {
    setBlockedApps(prev => [...prev, app.appId]);
    console.log(`Added ${app.displayName} to block list with ${mode} mode`);
  };

  const handleAddToFavorites = (app: AppSummary) => {
    console.log(`Added ${app.displayName} to favorites`);
  };

  return (
    <div className="p-4 max-w-2xl">
      <AppSearch 
        onAddToBlockList={handleAddToBlockList}
        onAddToFavorites={handleAddToFavorites}
        blockedApps={blockedApps}
      />
    </div>
  );
}