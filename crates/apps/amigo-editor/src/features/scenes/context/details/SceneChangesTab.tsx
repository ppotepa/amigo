import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import type { WorkspaceRuntimeServices } from "../../../../main-window/workspaceRuntimeServices";
import type { SceneContextModel } from "../sceneContextTypes";

export function SceneChangesTab({
  model,
  services,
}: {
  model: SceneContextModel;
  services: WorkspaceRuntimeServices;
}) {
  const changes = services.sceneChanges ?? {
    dirty: model.changes.dirty,
    summary: model.changes.summary,
    changedFiles: [],
    undoAvailable: false,
    redoAvailable: false,
  };

  return (
    <div className="scene-detail-tab">
      <section className="scene-detail-section">
      <ContextRow title="Dirty" subtitle={changes.dirty ? "yes" : "no"} />
      <ContextRow title="Summary" subtitle={changes.summary} />
      <ContextRow
        title="Changed files"
        subtitle={changes.changedFiles.length ? changes.changedFiles.join(", ") : "none"}
      />
      <ContextRow title="Undo stack" subtitle={changes.undoAvailable ? "available" : "unavailable"} />
      <ContextRow title="Redo stack" subtitle={changes.redoAvailable ? "available" : "unavailable"} />
      </section>
    </div>
  );
}
