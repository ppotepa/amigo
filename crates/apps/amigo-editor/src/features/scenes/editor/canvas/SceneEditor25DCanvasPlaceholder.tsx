import { Layers3, MousePointer2, Move3D } from "lucide-react";
import type { SceneEditorCanvasProps } from "../sceneEditorTypes";

export function SceneEditor25DCanvasPlaceholder({
  scene,
  selectedEntityId,
  snapshot,
}: SceneEditorCanvasProps) {
  return (
    <div className="scene-editor-canvas scene-editor-placeholder-canvas scene-editor-25d-placeholder">
      <div className="scene-editor-placeholder-grid scene-editor-placeholder-grid-isometric" />
      <div className="scene-editor-placeholder-card">
        <span className="scene-editor-placeholder-icon">
          <Layers3 size={34} />
        </span>
        <h3>2.5D Scene Editor</h3>
        <p>
          This scene is routed to the 2.5D canvas path. Isometric picking, depth-aware selection
          and projected transform gizmos are isolated from the pure 2D editor.
        </p>
        <div className="scene-editor-placeholder-facts">
          <span>Scene</span>
          <strong>{scene.documentPath || scene.id}</strong>
          <span>Selected</span>
          <strong>{selectedEntityId ?? "none"}</strong>
          <span>Objects</span>
          <strong>{snapshot?.objects.length ?? 0}</strong>
        </div>
        <div className="scene-editor-placeholder-steps">
          <span><MousePointer2 size={13} /> projected hit-test</span>
          <span><Move3D size={13} /> depth-aware gizmos</span>
          <span><Layers3 size={13} /> layer/depth inspector</span>
        </div>
      </div>
    </div>
  );
}
