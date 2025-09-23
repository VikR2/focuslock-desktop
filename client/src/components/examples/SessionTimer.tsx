import { useState } from "react";
import SessionTimer from '../SessionTimer';

export default function SessionTimerExample() {
  const [isRunning, setIsRunning] = useState(false);
  const [duration] = useState(25 * 60); // 25 minutes

  const handleStart = () => {
    console.log('Session started');
    setIsRunning(true);
  };

  const handlePause = () => {
    console.log('Session paused');
    setIsRunning(false);
  };

  const handleStop = () => {
    console.log('Session stopped');
    setIsRunning(false);
  };

  return (
    <div className="p-4">
      <SessionTimer 
        durationSecs={duration}
        isRunning={isRunning}
        onStart={handleStart}
        onPause={handlePause}
        onStop={handleStop}
      />
    </div>
  );
}