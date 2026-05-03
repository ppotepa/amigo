import { Circle } from "lucide-react";
import { useEditorStore } from "../app/editorStore";

export function ActivityFooter() {
  const { state } = useEditorStore();
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const latestEvents = state.events.slice(0, 3);

  return (
    <div className="activity-footer">
      <span className="activity-status">
        <Circle size={10} fill="currentColor" />
        {runningTasks.length > 0 ? `${runningTasks.length} task(s) running` : "Ready"}
      </span>
      <div className="event-trail">
        {latestEvents.map((event, index) => (
          <span key={`${event.type}:${index}`}>{event.type}</span>
        ))}
      </div>
    </div>
  );
}
