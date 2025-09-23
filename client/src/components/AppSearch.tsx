import { useState, useEffect } from "react";
import { Search, Plus, Shield, FolderOpen } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import type { AppSummary, BlockMode } from "@shared/schema";

interface SearchResult extends AppSummary {
  isBlocked?: boolean;
  blockMode?: BlockMode;
}

interface AppSearchProps {
  onAddToBlockList: (app: AppSummary, mode: BlockMode) => void;
  onAddToFavorites: (app: AppSummary) => void;
  blockedApps?: string[];
}

export default function AppSearch({ 
  onAddToBlockList, 
  onAddToFavorites,
  blockedApps = []
}: AppSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // todo: remove mock functionality - In real app, would search actual installed apps
  const mockApps: SearchResult[] = [
    {
      appId: 'discord.exe',
      displayName: 'Discord',
      exeOrTarget: 'C:\\Users\\User\\AppData\\Local\\Discord\\Discord.exe',
      iconHint: 'discord',
    },
    {
      appId: 'chrome.exe',
      displayName: 'Google Chrome',
      exeOrTarget: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
      iconHint: 'chrome',
    },
    {
      appId: 'steam.exe',
      displayName: 'Steam',
      exeOrTarget: 'C:\\Program Files (x86)\\Steam\\steam.exe',
      iconHint: 'steam',
    },
    {
      appId: 'spotify.exe',
      displayName: 'Spotify',
      exeOrTarget: 'C:\\Users\\User\\AppData\\Roaming\\Spotify\\Spotify.exe',
      iconHint: 'spotify',
    },
    {
      appId: 'slack.exe',
      displayName: 'Slack',
      exeOrTarget: 'C:\\Users\\User\\AppData\\Local\\slack\\slack.exe',
      iconHint: 'slack',
    },
    {
      appId: 'teams.exe',
      displayName: 'Microsoft Teams',
      exeOrTarget: 'C:\\Users\\User\\AppData\\Local\\Microsoft\\Teams\\Teams.exe',
      iconHint: 'teams',
    },
    {
      appId: 'notepad.exe',
      displayName: 'Notepad',
      exeOrTarget: 'C:\\Windows\\System32\\notepad.exe',
      iconHint: 'notepad',
    },
    {
      appId: 'code.exe',
      displayName: 'Visual Studio Code',
      exeOrTarget: 'C:\\Users\\User\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe',
      iconHint: 'vscode',
    },
  ];

  useEffect(() => {
    if (query.length >= 2) {
      setIsLoading(true);
      
      // Simulate search delay
      const timer = setTimeout(() => {
        const filtered = mockApps.filter(app => 
          app.displayName.toLowerCase().includes(query.toLowerCase()) ||
          app.appId.toLowerCase().includes(query.toLowerCase())
        ).map(app => ({
          ...app,
          isBlocked: blockedApps.includes(app.appId)
        }));
        
        setResults(filtered.slice(0, 8)); // Limit to 8 results
        setIsLoading(false);
      }, 200);

      return () => clearTimeout(timer);
    } else {
      setResults([]);
      setIsLoading(false);
    }
  }, [query, blockedApps]);

  const getAppIcon = (appId: string) => {
    // todo: remove mock functionality - In real app, would show actual app icons
    const icons: Record<string, string> = {
      'discord.exe': '🎮',
      'chrome.exe': '🌐',
      'steam.exe': '🎯',
      'spotify.exe': '🎵',
      'slack.exe': '💬',
      'teams.exe': '👥',
      'notepad.exe': '📝',
      'code.exe': '💻',
    };
    return icons[appId] || '📱';
  };

  return (
    <div className="space-y-4">
      {/* Search Input */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-muted-foreground" />
        <Input
          placeholder="Search for apps to block (e.g., Discord, Chrome)..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="pl-10"
          data-testid="input-app-search"
        />
        {isLoading && (
          <div className="absolute right-3 top-1/2 transform -translate-y-1/2">
            <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
          </div>
        )}
      </div>

      {/* Search Results */}
      {query.length >= 2 && (
        <Card>
          <CardContent className="p-4">
            {results.length === 0 && !isLoading ? (
              <div className="text-center py-8 text-muted-foreground">
                <Search className="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p>No apps found for "{query}"</p>
                <p className="text-xs mt-1">Try searching for common apps like Discord, Chrome, or Steam</p>
              </div>
            ) : (
              <div className="space-y-2">
                <h4 className="text-sm font-medium text-muted-foreground mb-3">
                  Search Results ({results.length})
                </h4>
                {results.map((app) => (
                  <div
                    key={app.appId}
                    className="flex items-center justify-between p-3 rounded-md border border-border hover-elevate"
                  >
                    <div className="flex items-center space-x-3 flex-1">
                      <div className="w-8 h-8 flex items-center justify-center text-lg">
                        {getAppIcon(app.appId)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-2">
                          <h5 className="font-medium text-sm truncate">
                            {app.displayName}
                          </h5>
                          {app.isBlocked && (
                            <Badge variant="destructive" className="text-xs">
                              Blocked
                            </Badge>
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground truncate">
                          {app.exeOrTarget}
                        </p>
                      </div>
                    </div>

                    <div className="flex items-center space-x-2">
                      {!app.isBlocked && (
                        <>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => onAddToBlockList(app, 'soft')}
                            data-testid={`button-block-soft-${app.appId}`}
                          >
                            <Shield className="w-3 h-3 mr-1" />
                            Soft Block
                          </Button>
                          <Button
                            size="sm"
                            onClick={() => onAddToBlockList(app, 'hard')}
                            data-testid={`button-block-hard-${app.appId}`}
                          >
                            <Shield className="w-3 h-3 mr-1" />
                            Hard Block
                          </Button>
                        </>
                      )}
                      
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => onAddToFavorites(app)}
                        data-testid={`button-favorite-${app.appId}`}
                      >
                        <Plus className="w-3 h-3 mr-1" />
                        Pin
                      </Button>
                      
                      <Button
                        size="sm"
                        variant="ghost"
                        data-testid={`button-reveal-${app.appId}`}
                        onClick={() => console.log(`Reveal ${app.appId} in explorer`)}
                      >
                        <FolderOpen className="w-3 h-3" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Search Tips */}
      {query.length < 2 && (
        <div className="text-center py-8 text-muted-foreground">
          <Search className="w-12 h-12 mx-auto mb-4 opacity-50" />
          <h4 className="font-medium mb-2">Find Apps to Block</h4>
          <p className="text-sm mb-4">
            Search for applications you want to block during focus sessions
          </p>
          <div className="grid grid-cols-2 gap-2 max-w-sm mx-auto text-xs">
            <div className="bg-muted/50 rounded p-2">
              <strong>Soft Block:</strong> Minimize and show reminder
            </div>
            <div className="bg-muted/50 rounded p-2">
              <strong>Hard Block:</strong> Prevent app from running
            </div>
          </div>
        </div>
      )}
    </div>
  );
}