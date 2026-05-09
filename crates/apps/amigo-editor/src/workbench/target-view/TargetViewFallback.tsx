import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function TargetViewFallback(props: EditorComponentProps<WorkspaceRuntimeServices>) {
  const { services } = props;
  return (
    <div className="workbench-fallback">
      <p className="muted workspace-empty">No inspector content is available for the current target.</p>
      <p className="muted workspace-empty">
        Target: {services.currentEditorTarget?.descriptor.label ?? "unknown"}
      </p>
    </div>
  );
}
