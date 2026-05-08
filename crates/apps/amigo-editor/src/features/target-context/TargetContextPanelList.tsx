import type { TargetPanelComponent } from "../../editor-targets/editorTargetContextTypes";
import type { ResolvedEditorTarget } from "../../editor-targets/editorTargetTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

// @codemap anchor:target-context-panel-list domain:workspace role:renderer priority:P1 layer:app tags:editor-target,right-dock,profile
export function TargetContextPanelList({
  emptyLabel = "No context panels for this target.",
  panels,
  services,
  target,
}: {
  emptyLabel?: string;
  panels: TargetPanelComponent[];
  services: WorkspaceRuntimeServices;
  target: ResolvedEditorTarget;
}) {
  if (panels.length === 0) {
    return <p className="muted workspace-note">{emptyLabel}</p>;
  }

  return (
    <>
      {panels.map((Panel, index) => (
        <Panel
          key={`${target.descriptor.kind}:${Panel.displayName ?? Panel.name ?? "panel"}:${index}`}
          services={services}
          target={target}
        />
      ))}
    </>
  );
}
