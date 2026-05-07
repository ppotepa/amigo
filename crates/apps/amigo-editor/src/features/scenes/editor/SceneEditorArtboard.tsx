import type {
  EditorFrameDto,
} from "../../../api/dto";
import { PreviewArtboard } from "../../../ui/preview";
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
    <PreviewArtboard
      className="scene-editor-artboard"
      chrome={false}
      width={resolution.width}
      height={resolution.height}
      panX={viewport.panX}
      panY={viewport.panY}
      zoom={viewport.zoom}
    >
      <SceneEditorRenderLayer frame={frame} />
    </PreviewArtboard>
  );
}
