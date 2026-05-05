import { Box, Camera, MousePointer2, Move3D, Orbit } from "lucide-react";
import type { SceneEditorCanvasProps } from "../sceneEditorTypes";

export function SceneEditor3DCanvasPlaceholder({
  scene,
  selectedEntityId,
  snapshot,
}: SceneEditorCanvasProps) {
  return (
    <div className="scene-editor-canvas scene-editor-placeholder-canvas scene-editor-3d-placeholder">
      <div className="scene-editor-placeholder-grid scene-editor-placeholder-grid-perspective" />
      <div className="scene-editor-placeholder-card">
        <span className="scene-editor-placeholder-icon">
          <Box size={34} />
        </span>
        <h3>3D Scene Editor</h3>
        <p>
          This scene is routed to the 3D canvas path. Perspective camera controls, raycast picking
          and 3D transform gizmos are isolated from the 2D editor implementation.
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
          <span><Camera size={13} /> orbit camera</span>
          <span><MousePointer2 size={13} /> raycast picking</span>
          <span><Move3D size={13} /> 3D transform gizmos</span>
          <span><Orbit size={13} /> world/local axes</span>
        </div>
      </div>
    </div>
  );
}
