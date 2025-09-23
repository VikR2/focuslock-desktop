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

// Removed FavoriteApp interface - now handled by individual components

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
  
  // Mock favorites functionality removed - now handled by FavoritesBar component directly

  // Mock block rules functionality removed - now handled by RulesTable component directly

  const [settings, setSettings] = useState({
    strictMode: false,
    autostart: true,
    notificationCadence: 'normal',
    defaultBlockMode: 'soft' as 'hard' | 'soft',
    hotkeysEnabled: true,
  });


  const handleAddFavorite = () => {
    console.log('Add favorite clicked - would open search/selection dialog');
  };

  // Search handlers - now handled by individual components
  const handleAddToBlockList = (app: AppSummary, mode: BlockMode) => {
    console.log(`Added ${app.displayName} to block list with ${mode} mode`);
    // This will be handled by the AppSearch component directly
  };

  const handleAddToFavorites = (app: AppSummary) => {
    console.log(`Added ${app.displayName} to favorites`);
    // This will be handled by the AppSearch component directly
  };

  // Rules handlers - now handled by RulesTable component directly
  const handleDeleteRule = (id: string) => {
    console.log(`Deleted rule ${id}`);
  };

  const handleEditRule = (rule: BlockRule) => {
    console.log(`Edit rule for ${rule.appId}`);
  };

  const handleToggleRuleMode = (id: string, mode: BlockMode) => {
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

  // Blocked app IDs will be handled by AppSearch component directly

  return (
    <div className="flex h-screen w-full">
      <FavoritesBar 
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
                  blockedApps={[]}
                />
              </div>
            </div>
          )} />
          
          <Route path="/rules" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Block Rules</h1>
                <RulesTable 
                  rules={[]}
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