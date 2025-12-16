import { useState } from "react";
import SessionPanel from '../SessionPanel';

export default function SessionPanelExample() {
  const [selectedDuration, setSelectedDuration] = useState(25 * 60);

  return (
    <div className="p-4 max-w-md">
      <SessionPanel
        selectedDuration={selectedDuration}
        onDurationChange={setSelectedDuration}
      />
    </div>
  );
}