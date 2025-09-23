import { useState } from "react";
import SessionPanel from '../SessionPanel';

export default function SessionPanelExample() {
  const [selectedDuration, setSelectedDuration] = useState(25 * 60);
  const [sessionStatus, setSessionStatus] = useState<'idle' | 'running' | 'paused'>('idle');

  const handleStartSession = (duration: number) => {
    console.log(`Starting session for ${Math.floor(duration / 60)} minutes`);
    setSessionStatus('running');
    // In real app, this would start the actual timer
  };

  return (
    <div className="p-4 max-w-md">
      <SessionPanel 
        selectedDuration={selectedDuration}
        onDurationChange={setSelectedDuration}
        sessionStatus={sessionStatus}
        onStartSession={handleStartSession}
      />
    </div>
  );
}