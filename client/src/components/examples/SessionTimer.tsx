import SessionTimer from "../SessionTimer";

export default function SessionTimerExample() {
  return (
    <div className="p-4">
      <SessionTimer selectedDuration={25 * 60} />
    </div>
  );
}