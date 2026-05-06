import type { SceneEditorEntity, SceneEditorModel } from "./sceneEditorTypes";

export function SceneEditorGeometryDebugPanel({
  model,
  selectedEntity,
}: {
  model: SceneEditorModel;
  selectedEntity: SceneEditorEntity | null;
}) {
  if (!selectedEntity) return null;

  return (
    <div className="scene-editor-geometry-debug">
      <strong>Selected object geometry</strong>
      <dl>
        <dt>entityId</dt>
        <dd>{selectedEntity.id}</dd>

        <dt>placement</dt>
        <dd>{selectedEntity.placementKind}</dd>

        <dt>edit command</dt>
        <dd>{selectedEntity.editCommandKind}</dd>

        <dt>layout source</dt>
        <dd>{model.layoutSource}</dd>

        <dt>movable</dt>
        <dd>{selectedEntity.movable ? "yes" : "no"}</dd>

        <dt>locked</dt>
        <dd>{selectedEntity.locked ? selectedEntity.lockedReason ?? "yes" : "no"}</dd>

        <dt>transform</dt>
        <dd>
          x {round(selectedEntity.transform.x)}, y {round(selectedEntity.transform.y)}, rot {round(selectedEntity.transform.rotation)}
        </dd>

        <dt>selection bounds</dt>
        <dd>{formatBounds(selectedEntity.selectionBounds ?? selectedEntity.bounds)}</dd>

        <dt>render bounds</dt>
        <dd>{selectedEntity.renderBounds ? formatBounds(selectedEntity.renderBounds) : "none"}</dd>

        <dt>components</dt>
        <dd>{selectedEntity.componentTypes.join(", ") || "none"}</dd>
      </dl>
    </div>
  );
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}

function formatBounds(bounds: { x: number; y: number; width: number; height: number }): string {
  return `x ${round(bounds.x)}, y ${round(bounds.y)}, w ${round(bounds.width)}, h ${round(bounds.height)}`;
}
