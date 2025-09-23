import { useState } from "react";
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { 
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { 
  MoreHorizontal, 
  Shield, 
  ShieldOff, 
  Trash2, 
  Edit,
  Gamepad2,
  Globe,
  Play,
  Music,
  MessageCircle,
  Users,
  FileText,
  Code,
  Monitor,
  Loader2
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
    'file-text': FileText,
    'code': Code,
    'monitor': Monitor,
  };
  return iconMap[iconName] || Monitor;
};
import type { BlockRule, MatchKind, BlockMode } from "@shared/schema";

interface RulesTableProps {
  rules: BlockRule[];
  onDeleteRule: (id: string) => void;
  onEditRule: (rule: BlockRule) => void;
  onToggleMode: (id: string, mode: BlockMode) => void;
  isLoading?: boolean;
  isUpdating?: boolean;
}

export default function RulesTable({ 
  rules, 
  onDeleteRule, 
  onEditRule,
  onToggleMode,
  isLoading = false,
  isUpdating = false
}: RulesTableProps) {
  const getMatchKindLabel = (kind: MatchKind) => {
    const labels: Record<MatchKind, string> = {
      'exe': 'Executable',
      'package': 'Package',
      'lnk': 'Shortcut',
      'path': 'Path',
      'regex': 'Pattern'
    };
    return labels[kind];
  };

  const getModeVariant = (mode: BlockMode) => {
    return mode === 'hard' ? 'destructive' : 'secondary';
  };

  const getAppIcon = (appId: string) => {
    // Using Lucide icons for proper app representation
    const iconMapping: Record<string, string> = {
      'discord.exe': 'gamepad-2',
      'chrome.exe': 'globe',
      'steam.exe': 'play',
      'spotify.exe': 'music',
      'slack.exe': 'message-circle',
      'teams.exe': 'users',
      'notepad.exe': 'file-text',
      'code.exe': 'code',
    };
    return iconMapping[appId] || 'monitor';
  };

  // Show loading state
  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center">
            <Shield className="w-5 h-5 mr-2" />
            Block Rules
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8">
            <Loader2 className="w-8 h-8 mx-auto mb-4 animate-spin text-muted-foreground" />
            <p className="text-sm text-muted-foreground">Loading block rules...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  // Show empty state
  if (rules.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Block Rules</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-center py-8 text-muted-foreground">
            <Shield className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <h4 className="font-medium mb-2">No Block Rules</h4>
            <p className="text-sm">
              Add apps to your block list to start managing your focus sessions
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center">
          <Shield className="w-5 h-5 mr-2" />
          Block Rules ({rules.length})
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-12"></TableHead>
              <TableHead>Application</TableHead>
              <TableHead>Match Type</TableHead>
              <TableHead>Block Mode</TableHead>
              <TableHead className="w-16"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {rules.map((rule) => (
              <TableRow key={rule.id} className="hover-elevate">
                <TableCell>
                  <div className="w-6 h-6 flex items-center justify-center">
                    {(() => {
                      const IconComponent = getIconComponent(getAppIcon(rule.appId));
                      return <IconComponent className="w-4 h-4 text-foreground" />;
                    })()}
                  </div>
                </TableCell>
                <TableCell>
                  <div>
                    <div className="font-medium text-sm">
                      {rule.appId.replace('.exe', '')}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {rule.appId}
                    </div>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="text-xs">
                    {getMatchKindLabel(rule.matchKind as MatchKind)}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge 
                    variant={getModeVariant(rule.mode as BlockMode)}
                    className="text-xs capitalize"
                  >
                    {rule.mode} Block
                  </Badge>
                </TableCell>
                <TableCell>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button 
                        variant="ghost" 
                        size="icon"
                        disabled={isUpdating}
                        data-testid={`button-rule-menu-${rule.id}`}
                      >
                        {isUpdating ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <MoreHorizontal className="w-4 h-4" />
                        )}
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem 
                        onClick={() => onEditRule(rule)}
                        data-testid={`menu-edit-rule-${rule.id}`}
                      >
                        <Edit className="w-4 h-4 mr-2" />
                        Edit Rule
                      </DropdownMenuItem>
                      <DropdownMenuItem 
                        onClick={() => onToggleMode(
                          rule.id, 
                          rule.mode === 'hard' ? 'soft' : 'hard'
                        )}
                        data-testid={`menu-toggle-mode-${rule.id}`}
                      >
                        {rule.mode === 'hard' ? (
                          <>
                            <ShieldOff className="w-4 h-4 mr-2" />
                            Change to Soft Block
                          </>
                        ) : (
                          <>
                            <Shield className="w-4 h-4 mr-2" />
                            Change to Hard Block
                          </>
                        )}
                      </DropdownMenuItem>
                      <DropdownMenuItem 
                        onClick={() => onDeleteRule(rule.id)}
                        className="text-destructive"
                        data-testid={`menu-delete-rule-${rule.id}`}
                      >
                        <Trash2 className="w-4 h-4 mr-2" />
                        Delete Rule
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}