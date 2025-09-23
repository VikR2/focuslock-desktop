import { useState, useEffect } from "react";
import { Play, Pause, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";

interface SessionTimerProps {
  durationSecs: number;
  isRunning: boolean;
  onStart: () => void;
  onPause: () => void;
  onStop: () => void;
}

export default function SessionTimer({ 
  durationSecs, 
  isRunning, 
  onStart, 
  onPause, 
  onStop 
}: SessionTimerProps) {
  const [remainingSecs, setRemainingSecs] = useState(durationSecs);

  useEffect(() => {
    setRemainingSecs(durationSecs);
  }, [durationSecs]);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (isRunning && remainingSecs > 0) {
      interval = setInterval(() => {
        setRemainingSecs(prev => {
          if (prev <= 1) {
            onPause();
            return 0;
          }
          return prev - 1;
        });
      }, 1000);
    }

    return () => clearInterval(interval);
  }, [isRunning, remainingSecs, onPause]);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const progressPercent = ((durationSecs - remainingSecs) / durationSecs) * 100;
  const isLowTime = remainingSecs <= 300; // 5 minutes

  return (
    <div className="flex flex-col items-center space-y-6 p-8">
      {/* Circular Timer Display */}
      <div className="relative flex items-center justify-center">
        <div className="w-48 h-48 rounded-full border-8 border-muted relative overflow-hidden">
          <div 
            className="absolute inset-0 bg-primary opacity-20 transform-gpu transition-transform duration-1000"
            style={{
              clipPath: `polygon(50% 50%, 50% 0%, ${50 + 50 * Math.cos((progressPercent / 100 * 360 - 90) * Math.PI / 180)}% ${50 + 50 * Math.sin((progressPercent / 100 * 360 - 90) * Math.PI / 180)}%, 50% 50%)`
            }}
          />
        </div>
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center">
            <div className={`text-4xl font-light ${isLowTime ? 'text-destructive animate-pulse' : 'text-foreground'}`}>
              {formatTime(remainingSecs)}
            </div>
            <div className="text-sm text-muted-foreground mt-1">
              {isRunning ? 'Focus Time' : 'Ready'}
            </div>
          </div>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="w-full max-w-md">
        <Progress value={progressPercent} className="h-2" />
        <div className="flex justify-between text-xs text-muted-foreground mt-1">
          <span>0:00</span>
          <span>{formatTime(durationSecs)}</span>
        </div>
      </div>

      {/* Control Buttons */}
      <div className="flex items-center space-x-4">
        {!isRunning ? (
          <Button 
            size="lg" 
            onClick={onStart} 
            data-testid="button-start-session"
            disabled={remainingSecs === 0}
          >
            <Play className="w-5 h-5 mr-2" />
            Start Focus
          </Button>
        ) : (
          <Button 
            size="lg" 
            variant="secondary" 
            onClick={onPause}
            data-testid="button-pause-session"
          >
            <Pause className="w-5 h-5 mr-2" />
            Pause
          </Button>
        )}
        
        <Button 
          size="lg" 
          variant="outline" 
          onClick={onStop}
          data-testid="button-stop-session"
          disabled={!isRunning && remainingSecs === durationSecs}
        >
          <Square className="w-5 h-5 mr-2" />
          Reset
        </Button>
      </div>

      {/* Session Info */}
      {isRunning && (
        <div className="text-center text-sm text-muted-foreground">
          <p>Focus session in progress</p>
          <p>Stay focused and avoid distractions!</p>
        </div>
      )}
    </div>
  );
}