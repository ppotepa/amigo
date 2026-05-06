import type {
  EditorFrameDto,
} from "../../../api/dto";
import type {
  SceneEditorResolution,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import { SceneEditorRenderLayer } from "./SceneEditorRenderLayer";

export function SceneEditorArtboard({
  frame,
  resolution,
  viewport,
}: {
  frame?: EditorFrameDto | null;
  resolution: SceneEditorResolution;
  viewport: SceneEditorViewportState;
}) {
  return (
    <div
      className="scene-editor-artboard"
      style={{
        width: resolution.width,
        height: resolution.height,
        transform: `translate(${viewport.panX}px, ${viewport.panY}px) scale(${viewport.zoom})`,
      }}
    >
      <SceneEditorRenderLayer frame={frame} />
    </div>
  );
}
