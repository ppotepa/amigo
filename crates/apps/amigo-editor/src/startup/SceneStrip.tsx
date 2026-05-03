import { FileCode2, ImageOff } from "lucide-react";
import type { EditorSceneSummaryDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";
import { previewImageSrc } from "./EngineSlideshowPreview";

export function SceneStrip({ modId, scenes, selectedSceneId }: { modId: string; scenes: EditorSceneSummaryDto[]; selectedSceneId: string | null }) {
  const { selectScene, state } = useEditorStore();

  return (
    <div className="scene-strip" aria-label="Scenes">
      {scenes.map((scene) => {
        const preview = state.previews[`${modId}:${scene.id}`];
        const thumbnail = previewImageSrc(preview?.imageUrl);
        return (
          <button key={scene.id} type="button" className={`scene-thumb interactive ${selectedSceneId === scene.id ? "selected" : ""}`} onClick={() => void selectScene(scene)}>
            <div className="scene-thumb-image">
              {thumbnail ? <img src={thumbnail} alt="" /> : <ImageOff size={20} />}
            </div>
            <div className="scene-thumb-copy">
              <strong>{scene.label}</strong>
              <span className="badge badge-muted">
                <FileCode2 size={12} />
                {scene.status}
              </span>
            </div>
          </button>
        );
      })}
    </div>
  );
}
