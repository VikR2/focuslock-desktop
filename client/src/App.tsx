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
import { Clock, Search, Shield, Settings, Monitor, ScrollText } from "lucide-react";

// Components
import SessionTimer from "@/components/SessionTimer";
import SessionPanel from "@/components/SessionPanel";
import FavoritesBar from "@/components/FavoritesBar";
import AppSearch from "@/components/AppSearch";
import RulesTable from "@/components/RulesTable";
import SettingsPanel from "@/components/SettingsPanel";
import ThemeToggle from "@/components/ThemeToggle";
import Logs from "@/pages/Logs";
import BlockedAppsList from "@/components/BlockedAppsList";
import { LogsProvider } from "@/contexts/LogsContext";

// Hooks
import { useBlockRules, useRemoveBlockRule, useUpdateBlockRule, useAddBlockRule } from "@/hooks/useBlockRules";
import { useSettings, useSetSetting, useSaveSettings } from "@/hooks/useSettings";
import { useAddFavorite } from "@/hooks/useFavorites";

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
      title: "Block Rules",
      url: "/rules",
      icon: Shield,
      id: "rules",
    },
    {
      title: "Logs",
      url: "/logs",
      icon: ScrollText,
      id: "logs",
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
  
  // API hooks for block rules
  const { data: blockRules = [], isLoading: rulesLoading } = useBlockRules();
  const removeBlockRuleMutation = useRemoveBlockRule();
  const updateBlockRuleMutation = useUpdateBlockRule();
  const addBlockRuleMutation = useAddBlockRule();
  
  // API hooks for favorites
  const addFavoriteMutation = useAddFavorite();
  
  // API hooks for settings  
  const { data: settings, isLoading: settingsLoading, error: settingsError } = useSettings();
  const setSettingMutation = useSetSetting();
  const saveSettingsMutation = useSaveSettings();
  
  // Mock favorites functionality removed - now handled by FavoritesBar component directly


  const handleAddFavorite = () => {
    console.log('Add favorite clicked - would open search/selection dialog');
  };

  // Real API handlers for app search actions
  const handleAddToBlockList = async (app: AppSummary, mode: BlockMode) => {
    try {
      await addBlockRuleMutation.mutateAsync({
        appId: app.appId,
        matchKind: 'exe' as const,
        mode: mode,
      });
      console.log(`Added ${app.displayName} to block list with ${mode} mode`);
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to add block rule:', error);
    }
  };

  const handleAddToFavorites = async (app: AppSummary) => {
    try {
      await addFavoriteMutation.mutateAsync({
        appId: app.appId,
        displayName: app.displayName,
        iconHint: app.iconHint,
      });
      console.log(`Added ${app.displayName} to favorites`);
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to add favorite:', error);
    }
  };

  // Real API handlers for rules
  const handleDeleteRule = async (id: string) => {
    try {
      await removeBlockRuleMutation.mutateAsync(id);
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to delete rule:', error);
    }
  };

  const handleEditRule = (rule: BlockRule) => {
    // TODO: Implement edit modal/form when needed
    console.log(`Edit rule for ${rule.appId}`);
  };

  const handleToggleRuleMode = async (id: string, mode: BlockMode) => {
    try {
      await updateBlockRuleMutation.mutateAsync({
        id,
        updates: { mode },
      });
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to toggle rule mode:', error);
    }
  };

  // Real API handlers for settings
  const handleSettingChange = async (key: string, value: any) => {
    try {
      await setSettingMutation.mutateAsync({ key, value });
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to update setting:', error);
    }
  };

  const handleSaveSettings = async () => {
    try {
      await saveSettingsMutation.mutateAsync(settings);
    } catch (error) {
      // Error handling is already done in the hook's onError
      console.error('Failed to save settings:', error);
    }
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
              <div className="max-w-4xl mx-auto space-y-6">
                {/* Compact Search Bar */}
                <div className="w-full max-w-md mx-auto">
                  <AppSearch 
                    onAddToBlockList={handleAddToBlockList}
                    onAddToFavorites={handleAddToFavorites}
                    blockedApps={blockRules.map(rule => rule.appId)}
                    compact={true}
                  />
                </div>
                
                {/* Timer and Session Panel */}
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  <div className="space-y-6">
                    <div className="flex justify-center">
                      <SessionTimer 
                        selectedDuration={selectedDuration}
                      />
                    </div>
                    <BlockedAppsList />
                  </div>
                  <div className="flex justify-center">
                    <SessionPanel 
                      selectedDuration={selectedDuration}
                      onDurationChange={setSelectedDuration}
                    />
                  </div>
                </div>
              </div>
            </div>
          )} />
          
          <Route path="/rules" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Block Rules</h1>
                <RulesTable 
                  rules={blockRules as BlockRule[]}
                  onDeleteRule={handleDeleteRule}
                  onEditRule={handleEditRule}
                  onToggleMode={handleToggleRuleMode}
                  isLoading={rulesLoading}
                  isUpdating={removeBlockRuleMutation.isPending || updateBlockRuleMutation.isPending}
                />
              </div>
            </div>
          )} />
          
          <Route path="/logs" component={Logs} />
          
          <Route path="/settings" component={() => (
            <div className="flex-1 p-6">
              <div className="max-w-4xl mx-auto">
                <h1 className="text-2xl font-semibold mb-6">Settings</h1>
                <SettingsPanel 
                  settings={settings}
                  onSettingChange={handleSettingChange}
                  onSave={handleSaveSettings}
                  isLoading={settingsLoading}
                  isSaving={setSettingMutation.isPending || saveSettingsMutation.isPending}
                  error={settingsError as Error | null}
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
        <LogsProvider>
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
        </LogsProvider>
      </TooltipProvider>
    </QueryClientProvider>
  );
}