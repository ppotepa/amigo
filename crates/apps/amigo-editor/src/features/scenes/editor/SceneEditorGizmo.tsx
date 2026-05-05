import type { SceneEditorEntity } from "./sceneEditorTypes";

export function SceneEditorGizmo({ entity }: { entity: SceneEditorEntity }) {
  return (
    <div
      className="scene-editor-gizmo"
      style={{
        left: entity.bounds.x,
        top: entity.bounds.y,
        width: entity.bounds.width,
        height: entity.bounds.height,
      }}
    >
      <span className="scene-editor-gizmo-handle scene-editor-gizmo-handle-nw" />
      <span className="scene-editor-gizmo-handle scene-editor-gizmo-handle-ne" />
      <span className="scene-editor-gizmo-handle scene-editor-gizmo-handle-sw" />
      <span className="scene-editor-gizmo-handle scene-editor-gizmo-handle-se" />
      <span className="scene-editor-gizmo-label">{entity.name}</span>
    </div>
  );
}
