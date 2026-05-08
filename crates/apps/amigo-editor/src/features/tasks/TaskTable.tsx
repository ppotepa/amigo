import { CheckCircle2, Clock, Loader2, XCircle } from "lucide-react";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { semanticIconClass, toneForStatus } from "../../theme/semanticColorRegistry";

export function TaskTable({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const tasks = services.tasks ?? [];
  return (
    <table className="workspace-table">
      <tbody>
        {tasks.slice(0, 12).map((task) => (
          <tr key={task.id}>
            <td>
              <span className={`badge ${taskStatusBadgeClass(task.status)}`}>
                {taskStatusIcon(task.status)}
                {task.status}
              </span>
            </td>
            <td><code>{task.id}</code></td>
            <td>{task.label}</td>
            <td>{task.progress != null ? `${Math.round(task.progress * 100)}%` : formatTaskTime(task.startedAt)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function taskStatusIcon(status: string) {
  const className = semanticIconClass(toneForStatus(status));
  if (status === "failed") return <XCircle size={12} className={className} />;
  if (status === "running") return <Loader2 size={12} className={className} />;
  if (status === "pending") return <Clock size={12} className="semantic-icon neutral" />;
  return <CheckCircle2 size={12} className={className} />;
}

const TASK_STATUS_BADGE_CLASS = {
  failed: "badge-error",
  running: "badge-info",
  finished: "badge-valid",
  pending: "badge-muted",
} as const;

function taskStatusBadgeClass(status: string): string {
  return isKnownTaskStatus(status) ? TASK_STATUS_BADGE_CLASS[status] : "badge-valid";
}

function isKnownTaskStatus(status: string): status is keyof typeof TASK_STATUS_BADGE_CLASS {
  return status in TASK_STATUS_BADGE_CLASS;
}

function formatTaskTime(value: number): string {
  return new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}
