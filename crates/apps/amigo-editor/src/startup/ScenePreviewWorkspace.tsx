import { Pause, Play, RefreshCcw, ScanSearch } from "lucide-react";
import type { ScenePreviewDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";
import { EngineSlideshowPreview } from "./EngineSlideshowPreview";
import { SceneStrip } from "./SceneStrip";

function previewKey(modId: string, sceneId: string): string {
  return `${modId}:${sceneId}`;
}

export function ScenePreviewWorkspace() {
  const { state, regeneratePreview, setPreviewPlaying } = useEditorStore();
  const details = state.modDetails;
  const scene = details?.scenes.find((item) => item.id === state.selectedSceneId);
  const selectedModId = state.selectedModId;
  const preview = selectedModId && scene ? state.previews[previewKey(selectedModId, scene.id)] : undefined;
  const task = selectedModId && scene ? state.tasks[`preview:${selectedModId}:${scene.id}`] : undefined;
  const isRendering = task?.status === "running";

  return (
    <section className="panel preview-workspace">
      <div className="panel-title-row">
        <div>
          <h2>Scene Preview</h2>
          <p>{scene ? scene.label : "Select a scene to preview."}</p>
        </div>
        <span className="pill cyan">5 FPS cap</span>
      </div>

      <div className="preview-toolbar">
        <button type="button" className="button button-tool" onClick={() => setPreviewPlaying(!state.previewPlaying)} disabled={!preview || preview.status !== "ready"}>
          {state.previewPlaying ? <Pause size={15} /> : <Play size={15} />}
          {state.previewPlaying ? "Pause" : "Play"}
        </button>
        <button type="button" className="button button-tool" disabled={!selectedModId || !scene || isRendering} onClick={() => selectedModId && scene ? void regeneratePreview(selectedModId, scene.id, true) : undefined}>
          <RefreshCcw size={15} />
          Regenerate
        </button>
      </div>

      <PreviewCanvas preview={preview} isRendering={Boolean(isRendering)} playing={state.previewPlaying} />

      {isRendering ? (
        <div className="preview-progress">
          <span style={{ width: `${Math.round((task?.progress ?? 0) * 100)}%` }} />
        </div>
      ) : null}

      {scene ? (
        <div className="selected-scene-meta">
          <strong>{scene.label}</strong>
          <span>{scene.documentPath}</span>
        </div>
      ) : null}

      {details ? <SceneStrip modId={details.id} scenes={details.scenes} selectedSceneId={state.selectedSceneId} /> : null}
    </section>
  );
}

function PreviewCanvas({ preview, isRendering, playing }: { preview?: ScenePreviewDto; isRendering: boolean; playing: boolean }) {
  if (isRendering) {
    return (
      <div className="preview-canvas preview-loading">
        <div className="spinner" />
        <strong>Rendering preview...</strong>
        <span>Generating 5 FPS preview frames.</span>
      </div>
    );
  }

  if (!preview) {
    return (
      <div className="preview-canvas preview-empty">
        <ScanSearch size={42} />
        <strong>No preview requested</strong>
        <span>Select a scene or press Regenerate.</span>
      </div>
    );
  }

  if (preview.status === "ready" && preview.frameUrls.length > 0) {
    return <EngineSlideshowPreview preview={preview} playing={playing} />;
  }

  return (
    <div className={`preview-canvas preview-${preview.status}`}>
      <strong>{preview.status}</strong>
      {preview.diagnostics.map((diagnostic) => (
        <p key={`${diagnostic.code}:${diagnostic.path ?? ""}`}>{diagnostic.message}</p>
      ))}
    </div>
  );
}
