import { DiagnosticsList } from "../../startup/DiagnosticsList";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function DiagnosticsPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <DiagnosticsList diagnostics={services.allProblems ?? []} />;
}
