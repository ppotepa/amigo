import type { ResolvedEditorTarget } from "../../../../editor-targets";
import type { WorkspaceRuntimeServices } from "../../../../main-window/workspaceRuntimeServices";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import { GenericPropertiesPanel } from "../../../metadata/GenericPropertiesPanel";
import type { SceneContextModel } from "../sceneContextTypes";

export type ScenePropertiesTabProps = {
  model: SceneContextModel;
  services: WorkspaceRuntimeServices;
};

export function ScenePropertiesTab({
  model,
  services,
}: ScenePropertiesTabProps) {
  const target = services.currentDetailTarget ?? services.currentEditorTarget ?? null;

  if (!target) {
    return (
      <div className="scene-detail-tab">
        <p className="muted workspace-note">Select a scene item to inspect its properties.</p>
      </div>
    );
  }

  return (
    <div className="scene-detail-tab">
      <section className="scene-detail-section scene-detail-target-summary">
        <ContextRow title="Target" subtitle={target.descriptor.label} badge={target.ref.kind} />
      </section>
      {selectedComponentFromTarget(target) ? (
        <GenericPropertiesPanel
          metadata={services.metadataCatalog}
          component={selectedComponentFromTarget(target)}
          assetRegistry={services.assetRegistry}
          onRequestAssetAssign={(request) => {
            services.requestAssignAssetRef?.({
              target: target.ref,
              path: request.path,
              assetKey: request.assetKey,
            });
          }}
        />
      ) : (
        <SceneTargetFallbackProperties target={target} sceneLabel={model.scene.label || model.scene.id} />
      )}
    </div>
  );
}

function selectedComponentFromTarget(target: ResolvedEditorTarget) {
  if (target.selection.kind !== "component") return null;
  return target.selection.component;
}

function SceneTargetFallbackProperties({
  sceneLabel,
  target,
}: {
  sceneLabel: string;
  target: ResolvedEditorTarget;
}) {
  return (
    <div className="scene-properties-fallback">
      <ContextRow title="Kind" subtitle={target.ref.kind} />
      <ContextRow title="Scene" subtitle={sceneLabel} />
    </div>
  );
}
