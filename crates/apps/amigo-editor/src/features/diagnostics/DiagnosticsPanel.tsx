import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { ProblemsTable } from "./ProblemsTable";

// @codemap anchor:diagnostics-panel-target-wiring domain:workspace role:tree priority:P1 layer:app tags:diagnostics,editor-target,right-dock
export function DiagnosticsPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <ProblemsTable services={services} />;
}
