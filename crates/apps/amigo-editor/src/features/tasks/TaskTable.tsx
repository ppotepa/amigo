import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function TaskTable({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const tasks = services.tasks ?? [];
  return (
    <table className="workspace-table">
      <tbody>
        {tasks.slice(0, 12).map((task) => (
          <tr key={task.id}>
            <td><span className={`badge ${taskStatusBadgeClass(task.status)}`}>{task.status}</span></td>
            <td><code>{task.id}</code></td>
            <td>{task.label}</td>
            <td>{task.progress != null ? `${Math.round(task.progress * 100)}%` : formatTaskTime(task.startedAt)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

const TASK_STATUS_BADGE_CLASS: Record<string, string> = {
  failed: "badge-error",
  running: "badge-info",
  finished: "badge-valid",
  pending: "badge-muted",
};

function taskStatusBadgeClass(status: string): string {
  return TASK_STATUS_BADGE_CLASS[status] ?? "badge-valid";
}

function formatTaskTime(value: number): string {
  return new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}
