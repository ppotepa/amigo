import { Box, Square } from "lucide-react";
import type { SceneEditorCanvasKind } from "../sceneEditorTypes";

export function SceneCanvasKindBadge({
  kind,
}: {
  kind: SceneEditorCanvasKind;
}) {
  if (kind === "3d") {
    return (
      <div className="scene-canvas-kind-badge scene-canvas-kind-badge-3d" aria-label="3D scene canvas">
        <Box size={18} />
        <span>3D</span>
      </div>
    );
  }

  if (kind === "2.5d") {
    return (
      <div className="scene-canvas-kind-badge scene-canvas-kind-badge-25d" aria-label="2.5D scene canvas">
        <span className="scene-canvas-kind-iso" />
        <span>2.5D</span>
      </div>
    );
  }

  return (
    <div className="scene-canvas-kind-badge scene-canvas-kind-badge-2d" aria-label="2D scene canvas">
      <Square size={18} />
      <span>2D</span>
    </div>
  );
}
