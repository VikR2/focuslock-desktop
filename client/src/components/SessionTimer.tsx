import { useState, useEffect } from "react";
import { Play, Pause, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { useCurrentSession, useCreateSession, useUpdateSession } from "@/hooks/useSession";
import type { Session } from "@shared/schema";

interface SessionTimerProps {
  selectedDuration: number;
}

export default function SessionTimer({ selectedDuration }: SessionTimerProps) {
  const { data: currentSession, isLoading: isLoadingSession } = useCurrentSession();
  const createSession = useCreateSession();
  const updateSession = useUpdateSession();
  
  const [remainingSecs, setRemainingSecs] = useState(selectedDuration);

  // Calculate remaining time based on current session
  useEffect(() => {
    if (currentSession && currentSession.status === 'running') {
      const startTime = new Date(currentSession.startTime!).getTime();
      const now = Date.now();
      const elapsed = Math.floor((now - startTime) / 1000);
      const remaining = Math.max(0, currentSession.durationSecs - elapsed);
      setRemainingSecs(remaining);
    } else if (currentSession && currentSession.status === 'paused') {
      setRemainingSecs(currentSession.remainingSecs || selectedDuration);
    } else {
      setRemainingSecs(selectedDuration);
    }
  }, [currentSession, selectedDuration]);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    
    if (currentSession?.status === 'running' && remainingSecs > 0) {
      interval = setInterval(() => {
        setRemainingSecs(prev => {
          if (prev <= 1) {
            // Auto-complete session when time runs out
            if (currentSession?.id) {
              updateSession.mutate({
                id: currentSession.id,
                updates: { 
                  status: 'completed',
                  endTime: new Date().toISOString(),
                  remainingSecs: 0
                }
              });
            }
            return 0;
          }
          
          // Update remaining time in backend every 10 seconds
          if (prev % 10 === 0 && currentSession?.id) {
            updateSession.mutate({
              id: currentSession.id,
              updates: { remainingSecs: prev - 1 }
            });
          }
          
          return prev - 1;
        });
      }, 1000);
    }

    return () => clearInterval(interval);
  }, [currentSession, remainingSecs, updateSession]);

  const handleStartSession = async () => {
    if (currentSession && currentSession.status === 'paused') {
      // Resume paused session
      updateSession.mutate({
        id: currentSession.id,
        updates: { 
          status: 'running',
          startTime: new Date().toISOString()
        }
      });
    } else {
      // Create new session
      createSession.mutate({
        durationSecs: selectedDuration,
        remainingSecs: selectedDuration,
        status: 'running',
        startTime: new Date().toISOString(),
      });
    }
  };

  const handlePauseSession = () => {
    if (currentSession?.id) {
      updateSession.mutate({
        id: currentSession.id,
        updates: { 
          status: 'paused',
          remainingSecs
        }
      });
    }
  };

  const handleStopSession = () => {
    if (currentSession?.id) {
      updateSession.mutate({
        id: currentSession.id,
        updates: { 
          status: 'canceled',
          endTime: new Date().toISOString(),
          remainingSecs: selectedDuration
        }
      });
    }
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const durationSecs = currentSession?.durationSecs || selectedDuration;
  const progressPercent = ((durationSecs - remainingSecs) / durationSecs) * 100;
  const isLowTime = remainingSecs <= 300; // 5 minutes
  const isRunning = currentSession?.status === 'running';
  const isLoading = isLoadingSession || createSession.isPending || updateSession.isPending;

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
            onClick={handleStartSession} 
            data-testid="button-start-session"
            disabled={remainingSecs === 0 || isLoading}
          >
            <Play className="w-5 h-5 mr-2" />
            {currentSession?.status === 'paused' ? 'Resume' : 'Start Focus'}
          </Button>
        ) : (
          <Button 
            size="lg" 
            variant="secondary" 
            onClick={handlePauseSession}
            data-testid="button-pause-session"
            disabled={isLoading}
          >
            <Pause className="w-5 h-5 mr-2" />
            Pause
          </Button>
        )}
        
        <Button 
          size="lg" 
          variant="outline" 
          onClick={handleStopSession}
          data-testid="button-stop-session"
          disabled={(!isRunning && remainingSecs === durationSecs) || isLoading}
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