import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Clock, Settings } from "lucide-react";

interface SessionPanelProps {
  selectedDuration: number;
  onDurationChange: (duration: number) => void;
  onStartSession: (duration: number) => void;
}

const PRESET_DURATIONS = [
  { label: '25 min', value: 25 * 60, description: 'Pomodoro' },
  { label: '45 min', value: 45 * 60, description: 'Deep Work' },
  { label: '60 min', value: 60 * 60, description: 'Extended' },
];

export default function SessionPanel({ 
  selectedDuration, 
  onDurationChange,
  onStartSession 
}: SessionPanelProps) {
  const [customMinutes, setCustomMinutes] = useState(25);
  const [showCustom, setShowCustom] = useState(false);

  const handlePresetSelect = (duration: number) => {
    onDurationChange(duration);
    setShowCustom(false);
  };

  const handleCustomSubmit = () => {
    const duration = customMinutes * 60;
    onDurationChange(duration);
    setShowCustom(false);
  };

  const isCustomDuration = !PRESET_DURATIONS.some(preset => preset.value === selectedDuration);

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center">
          <Clock className="w-5 h-5 mr-2" />
          Focus Session
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Duration Presets */}
        <div className="space-y-3">
          <h4 className="text-sm font-medium text-muted-foreground">Duration Presets</h4>
          <div className="grid grid-cols-3 gap-2">
            {PRESET_DURATIONS.map((preset) => (
              <Button
                key={preset.value}
                variant={selectedDuration === preset.value ? "default" : "outline"}
                size="sm"
                onClick={() => handlePresetSelect(preset.value)}
                data-testid={`button-preset-${preset.value}`}
                className="flex-col h-auto py-3"
                disabled={false}
              >
                <span className="font-semibold">{preset.label}</span>
                <span className="text-xs opacity-75">{preset.description}</span>
              </Button>
            ))}
          </div>
        </div>

        {/* Custom Duration */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-medium text-muted-foreground">Custom Duration</h4>
            {isCustomDuration && (
              <Badge variant="secondary" className="text-xs">
                {Math.floor(selectedDuration / 60)} min
              </Badge>
            )}
          </div>
          
          {showCustom ? (
            <div className="flex items-center space-x-2">
              <Input
                type="number"
                min="1"
                max="180"
                value={customMinutes}
                onChange={(e) => setCustomMinutes(Number(e.target.value))}
                className="flex-1"
                placeholder="Minutes"
                data-testid="input-custom-duration"
              />
              <Button size="sm" onClick={handleCustomSubmit} data-testid="button-apply-custom">
                Apply
              </Button>
              <Button 
                size="sm" 
                variant="ghost" 
                onClick={() => setShowCustom(false)}
                data-testid="button-cancel-custom"
              >
                Cancel
              </Button>
            </div>
          ) : (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowCustom(true)}
              data-testid="button-custom-duration"
              disabled={false}
              className="w-full"
            >
              <Settings className="w-4 h-4 mr-2" />
              Set Custom Duration
            </Button>
          )}
        </div>

        {/* Start Session Button */}
        <Button
          size="lg"
          onClick={() => onStartSession(selectedDuration)}
          disabled={false}
          data-testid="button-start-focus-session"
          className="w-full"
        >
          Start {Math.floor(selectedDuration / 60)}-Minute Focus Session
        </Button>

        {/* Session Info */}
        <div className="text-xs text-muted-foreground text-center">
          <p>During your focus session, selected apps will be blocked</p>
          <p>Configure your block list to get started</p>
        </div>
      </CardContent>
    </Card>
  );
}