import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { TargetContextPanelList } from "./TargetContextPanelList";

// @codemap anchor:target-context-panel domain:workspace role:renderer priority:P1 layer:app tags:editor-target,right-dock,profile
export function TargetContextPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const target = services.currentEditorTarget ?? null;

  if (!target) {
    return (
      <div className="dock-scroll target-context-panel">
        <p className="muted workspace-empty">Select an editor target to inspect context.</p>
      </div>
    );
  }

  return (
    <div className="dock-scroll target-context-panel">
      <TargetContextPanelList
        panels={target.contextProfile.secondary}
        services={services}
        target={target}
      />
    </div>
  );
}
