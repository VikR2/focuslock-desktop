import { useState } from "react";
import { Switch, Route } from "wouter";
import { queryClient } from "./lib/queryClient";
import { QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "@/components/ui/toaster";
import { TooltipProvider } from "@/components/ui/tooltip";
import { 
  SidebarProvider, 
  SidebarTrigger,
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Clock, Search, Shield, Settings, Monitor } from "lucide-react";

// Components
import SessionTimer from "@/components/SessionTimer";
import SessionPanel from "@/components/SessionPanel";
import FavoritesBar from "@/components/FavoritesBar";
import AppSearch from "@/components/AppSearch";
import RulesTable from "@/components/RulesTable";
import SettingsPanel from "@/components/SettingsPanel";
import ThemeToggle from "@/components/ThemeToggle";

// Types
import type { 
  BlockRule, 
  BlockMode, 
  AppSummary,
  SessionStatus 
} from "@shared/schema";

interface FavoriteApp {
  id: string;
  appId: string;
  displayName: string;
  pinnedOrder: number | null;
  iconHint: string | null;
  isBlocked: boolean;
  blockMode: BlockMode;
}

function AppSidebar() {
  const menuItems = [
    {
      title: "Focus Timer",
      url: "/",
      icon: Clock,
      id: "timer",
    },
    {
      title: "Search Apps",
      url: "/search",
      icon: Search,
      id: "search",
    },
    {
      title: "Block Rules",
      url: "/rules",
      icon: Shield,
      id: "rules",
    },
    {
      title: "Settings",
      url: "/settings",
      icon: Settings,
      id: "settings",
    },
  ];

  return (
    <Sidebar>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>
            <div className="flex items-center">
              <Monitor className="w-4 h-4 mr-2" />
              FocusLock
            </div>
          </SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {menuItems.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <a href={item.url} data-testid={`nav-${item.id}`}>
                      <item.icon />
                      <span>{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}

function Router() {
  const [selectedDuration, setSelectedDuration] = useState(25 * 60); // 25 minutes
  
  // todo: remove mock functionality - In real app, would fetch from backend
  const [favorites, setFavorites] = useState<FavoriteApp[]>([
    {
      id: '1',
      appId: 'discord.exe',
      displayName: 'Discord',
      pinnedOrder: 1,
      iconHint: 'discord',
      isBlocked: true,
      blockMode: 'hard',
    },
    {
      id: '2', 
      appId: 'chrome.exe',
      displayName: 'Google Chrome',
      pinnedOrder: 2,
      iconHint: 'chrome',
      isBlocked: false,
      blockMode: 'soft',
    },
    {
      id: '3',
      appId: 'steam.exe', 
      displayName: 'Steam',
      pinnedOrder: 3,
      iconHint: 'steam',
      isBlocked: true,
      blockMode: 'soft',
    },
  ]);

  const [blockRules, setBlockRules] = useState<BlockRule[]>([
    {
      id: '1',
      appId: 'discord.exe',
      matchKind: 'exe',
      mode: 'hard',
    },
    {
      id: '3',
      appId: 'steam.exe',
      matchKind: 'exe',
      mode: 'soft',
    },
  ]);

  const [settings, setSettings] = useState({
    strictMode: false,
    autostart: true,
    notificationCadence: 'normal',
    defaultBlockMode: 'soft' as 'hard' | 'soft',
    hotkeysEnabled: true,
  });


  // Favorites handlers
  const handleToggleFavoriteBlock = (appId: string) => {
    setFavorites(prev => prev.map(fav => 
      fav.appId === appId ? { ...fav, isBlocked: !fav.isBlocked } : fav
    ));
    
    // Sync with block rules
    const favorite = favorites.find(f => f.appId === appId);
    if (favorite?.isBlocked) {
      // Remove from block rules
      setBlockRules(prev => prev.filter(rule => rule.appId !== appId));
    } else {
      // Add to block rules
      const newRule: BlockRule = {
        id: Date.now().toString(),
        appId,
        matchKind: 'exe',
        mode: favorite?.blockMode || 'soft',
      };
      setBlockRules(prev => [...prev, newRule]);
    }
    console.log(`Toggled block for ${appId}`);
  };

  const handleToggleFavoriteMode = (appId: string, mode: BlockMode) => {
    setFavorites(prev => prev.map(fav => 
      fav.appId === appId ? { ...fav, blockMode: mode } : fav
    ));
    
    // Sync with block rules
    setBlockRules(prev => prev.map(rule => 
      rule.appId === appId ? { ...rule, mode } : rule
    ));
    console.log(`Changed block mode for ${appId} to ${mode}`);
  };

  const handleRemoveFavorite = (id: string) => {
    setFavorites(prev => prev.filter(fav => fav.id !== id));
    console.log(`Removed favorite ${id}`);
  };

  const handleAddFavorite = () => {
    console.log('Add favorite clicked - would open search/selection dialog');
  };

  // Search handlers
  const handleAddToBlockList = (app: AppSummary, mode: BlockMode) => {
    const newRule: BlockRule = {
      id: Date.now().toString(),
      appId: app.appId,
      matchKind: 'exe',
      mode,
    };
    setBlockRules(prev => [...prev, newRule]);
    console.log(`Added ${app.displayName} to block list with ${mode} mode`);
  };

  const handleAddToFavorites = (app: AppSummary) => {
    const newFavorite: FavoriteApp = {
      id: Date.now().toString(),
      appId: app.appId,
      displayName: app.displayName,
      pinnedOrder: favorites.length + 1,
      iconHint: app.iconHint || null,
      isBlocked: false,
      blockMode: 'soft',
    };
    setFavorites(prev => [...prev, newFavorite]);
    console.log(`Added ${app.displayName} to favorites`);
  };

  // Rules handlers
  const handleDeleteRule = (id: string) => {
    setBlockRules(prev => prev.filter(rule => rule.id !== id));
    
    // Sync with favorites
    const rule = blockRules.find(r => r.id === id);
    if (rule) {
      setFavorites(prev => prev.map(fav => 
        fav.appId === rule.appId ? { ...fav, isBlocked: false } : fav
      ));
    }
    console.log(`Deleted rule ${id}`);
  };

  const handleEditRule = (rule: BlockRule) => {
    console.log(`Edit rule for ${rule.appId}`);
  };

  const handleToggleRuleMode = (id: string, mode: BlockMode) => {
    setBlockRules(prev => prev.map(rule => 
      rule.id === id ? { ...rule, mode } : rule
    ));
    
    // Sync with favorites
    const rule = blockRules.find(r => r.id === id);
    if (rule) {
      setFavorites(prev => prev.map(fav => 
        fav.appId === rule.appId ? { ...fav, blockMode: mode } : fav
      ));
    }
    console.log(`Toggled mode for rule ${id} to ${mode}`);
  };

  // Settings handlers
  const handleSettingChange = (key: string, value: any) => {
    setSettings(prev => ({ ...prev, [key]: value }));
    console.log(`Setting ${key} changed to:`, value);
  };

  const handleSaveSettings = () => {
    console.log('Settings saved:', settings);
  };

  const blockedAppIds = blockRules.map(rule => rule.appId);

  return (
    <div className="flex h-screen w-full">
      <FavoritesBar 
        favorites={favorites}
        onToggleBlock={handleToggleFavoriteBlock}
        onToggleMode={handleToggleFavoriteMode}
        onRemoveFavorite={handleRemoveFavorite}
        onAddFavorite={handleAddFavorite}
      />
      
      <div className="flex-1 flex flex-col">
        <Switch>
          <Route path="/" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div className="flex justify-center">
                  <SessionTimer 
                    selectedDuration={selectedDuration}
                  />
                </div>
                <div className="flex justify-center">
                  <SessionPanel 
                    selectedDuration={selectedDuration}
                    onDurationChange={setSelectedDuration}
                    onStartSession={(duration: number) => setSelectedDuration(duration)}
                  />
                </div>
              </div>
            </div>
          )} />
          
          <Route path="/search" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Search Applications</h1>
                <AppSearch 
                  onAddToBlockList={handleAddToBlockList}
                  onAddToFavorites={handleAddToFavorites}
                  blockedApps={blockedAppIds}
                />
              </div>
            </div>
          )} />
          
          <Route path="/rules" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Block Rules</h1>
                <RulesTable 
                  rules={blockRules}
                  onDeleteRule={handleDeleteRule}
                  onEditRule={handleEditRule}
                  onToggleMode={handleToggleRuleMode}
                />
              </div>
            </div>
          )} />
          
          <Route path="/settings" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Settings</h1>
                <SettingsPanel 
                  settings={settings}
                  onSettingChange={handleSettingChange}
                  onSave={handleSaveSettings}
                />
              </div>
            </div>
          )} />
        </Switch>
      </div>
    </div>
  );
}

export default function App() {
  const style = {
    "--sidebar-width": "20rem",
    "--sidebar-width-icon": "4rem",
  };

  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <SidebarProvider style={style as React.CSSProperties}>
          <div className="flex h-screen w-full">
            <AppSidebar />
            <div className="flex flex-col flex-1">
              <header className="flex items-center justify-between p-4 border-b border-border">
                <SidebarTrigger data-testid="button-sidebar-toggle" />
                <ThemeToggle />
              </header>
              <main className="flex-1 overflow-hidden">
                <Router />
              </main>
            </div>
          </div>
        </SidebarProvider>
        <Toaster />
      </TooltipProvider>
    </QueryClientProvider>
  );
}