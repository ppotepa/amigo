import type { ScenePreviewDto } from "../../../api/dto";
import { sceneEditorFrameUrl } from "./sceneEditorModel";

export function SceneEditorRenderLayer({ preview }: { preview?: ScenePreviewDto }) {
  const frameUrl = sceneEditorFrameUrl(preview);

  if (!frameUrl) {
    return (
      <div className="scene-editor-render-layer scene-editor-render-placeholder">
        <span>No engine frame yet</span>
      </div>
    );
  }

  return (
    <div className="scene-editor-render-layer">
      <img src={frameUrl} alt="Scene render" draggable={false} />
    </div>
  );
}
