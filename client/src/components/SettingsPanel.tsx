import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Separator } from "@/components/ui/separator";
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Settings, Shield, Bell, Keyboard, Play } from "lucide-react";

interface SettingsPanelProps {
  settings: {
    strictMode: boolean;
    autostart: boolean;
    notificationCadence: string;
    defaultBlockMode: 'hard' | 'soft';
    hotkeysEnabled: boolean;
  };
  onSettingChange: (key: string, value: any) => void;
  onSave: () => void;
}

export default function SettingsPanel({ 
  settings, 
  onSettingChange, 
  onSave 
}: SettingsPanelProps) {
  const [passphrase, setPassphrase] = useState("");
  const [confirmPassphrase, setConfirmPassphrase] = useState("");

  const notificationOptions = [
    { value: 'frequent', label: 'Frequent (15m, 10m, 5m, 1m)' },
    { value: 'normal', label: 'Normal (15m, 5m, 1m)' },
    { value: 'minimal', label: 'Minimal (5m, 1m)' },
    { value: 'none', label: 'None' },
  ];

  return (
    <Card className="w-full max-w-2xl">
      <CardHeader>
        <CardTitle className="flex items-center">
          <Settings className="w-5 h-5 mr-2" />
          FocusLock Settings
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        
        {/* Strict Mode Section */}
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-muted-foreground" />
            <h4 className="font-medium">Strict Mode</h4>
          </div>
          
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <Label htmlFor="strict-mode">Enable Strict Mode</Label>
              <p className="text-sm text-muted-foreground">
                Prevent quitting FocusLock during active sessions
              </p>
            </div>
            <Switch
              id="strict-mode"
              checked={settings.strictMode}
              onCheckedChange={(checked) => onSettingChange('strictMode', checked)}
              data-testid="switch-strict-mode"
            />
          </div>

          {settings.strictMode && (
            <div className="space-y-3 pl-6 border-l-2 border-muted">
              <div className="space-y-2">
                <Label htmlFor="passphrase">Passphrase (optional)</Label>
                <Input
                  id="passphrase"
                  type="password"
                  placeholder="Enter passphrase to exit strict mode"
                  value={passphrase}
                  onChange={(e) => setPassphrase(e.target.value)}
                  data-testid="input-passphrase"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="confirm-passphrase">Confirm Passphrase</Label>
                <Input
                  id="confirm-passphrase"
                  type="password"
                  placeholder="Confirm passphrase"
                  value={confirmPassphrase}
                  onChange={(e) => setConfirmPassphrase(e.target.value)}
                  data-testid="input-confirm-passphrase"
                />
              </div>
              {passphrase && passphrase !== confirmPassphrase && (
                <p className="text-sm text-destructive">Passphrases do not match</p>
              )}
            </div>
          )}
        </div>

        <Separator />

        {/* Notifications Section */}
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <Bell className="w-4 h-4 text-muted-foreground" />
            <h4 className="font-medium">Notifications</h4>
          </div>
          
          <div className="space-y-3">
            <Label htmlFor="notification-cadence">Notification Frequency</Label>
            <Select 
              value={settings.notificationCadence} 
              onValueChange={(value) => onSettingChange('notificationCadence', value)}
            >
              <SelectTrigger data-testid="select-notification-cadence">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {notificationOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <Separator />

        {/* Default Behavior Section */}
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-muted-foreground" />
            <h4 className="font-medium">Default Behavior</h4>
          </div>
          
          <div className="space-y-3">
            <Label htmlFor="default-block-mode">Default Block Mode</Label>
            <Select 
              value={settings.defaultBlockMode} 
              onValueChange={(value) => onSettingChange('defaultBlockMode', value)}
            >
              <SelectTrigger data-testid="select-default-block-mode">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="soft">Soft Block (minimize app, show reminder)</SelectItem>
                <SelectItem value="hard">Hard Block (prevent app from running)</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <Separator />

        {/* System Integration Section */}
        <div className="space-y-4">
          <div className="flex items-center space-x-2">
            <Play className="w-4 h-4 text-muted-foreground" />
            <h4 className="font-medium">System Integration</h4>
          </div>
          
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <Label htmlFor="autostart">Start with Windows</Label>
              <p className="text-sm text-muted-foreground">
                Automatically start FocusLock when you log in
              </p>
            </div>
            <Switch
              id="autostart"
              checked={settings.autostart}
              onCheckedChange={(checked) => onSettingChange('autostart', checked)}
              data-testid="switch-autostart"
            />
          </div>

          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <Label htmlFor="hotkeys">Global Hotkeys</Label>
              <p className="text-sm text-muted-foreground">
                Enable keyboard shortcuts to control sessions
              </p>
            </div>
            <Switch
              id="hotkeys"
              checked={settings.hotkeysEnabled}
              onCheckedChange={(checked) => onSettingChange('hotkeysEnabled', checked)}
              data-testid="switch-hotkeys"
            />
          </div>
        </div>

        <Separator />

        {/* Save Button */}
        <div className="flex justify-end">
          <Button onClick={onSave} data-testid="button-save-settings">
            Save Settings
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}