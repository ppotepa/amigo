import { Pause, Play, RefreshCcw, ScanSearch } from "lucide-react";
import type { ScenePreviewDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";
import { selectedModId, selectedSceneId } from "../app/selectionSelectors";
import { activePreview as resolveActivePreview, selectedScene as resolveSelectedScene } from "../app/store/editorSelectors";
import { EngineSlideshowPreview } from "./EngineSlideshowPreview";
import { SceneStrip } from "./SceneStrip";

export function ScenePreviewWorkspace() {
  const { state, regeneratePreview, setPreviewPlaying } = useEditorStore();
  const details = state.modDetails;
  const scene = resolveSelectedScene(state);
  const modId = selectedModId(state.selection);
  const preview = resolveActivePreview(details, scene?.id ?? null, state.previews);
  const task = modId && scene ? state.tasks[`preview:${modId}:${scene.id}`] : undefined;
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
        <button type="button" className="button button-tool" disabled={!modId || !scene || isRendering} onClick={() => modId && scene ? void regeneratePreview(modId, scene.id, true) : undefined}>
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

      {details ? <SceneStrip modId={details.id} scenes={details.scenes} selectedSceneId={scene?.id ?? selectedSceneId(state.selection)} /> : null}
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
