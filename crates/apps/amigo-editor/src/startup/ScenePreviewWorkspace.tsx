import { ScanSearch } from "lucide-react";
import type { ScenePreviewDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";
import { selectedModId } from "../app/selectionSelectors";
import { activePreview as resolveActivePreview, selectedScene as resolveSelectedScene } from "../app/store/editorSelectors";
import { DebugSourceOverlay, useDebugSourceEnabled } from "../debug/debugSource";
import { EngineSlideshowPreview } from "./EngineSlideshowPreview";

export function ScenePreviewWorkspace() {
  const { state } = useEditorStore();
  const showDebugSources = useDebugSourceEnabled();
  const details = state.modDetails;
  const scene = resolveSelectedScene(state);
  const modId = selectedModId(state.selection);
  const preview = resolveActivePreview(details, scene?.id ?? null, state.previews);
  const task = modId && scene ? state.tasks[`preview:${modId}:${scene.id}`] : undefined;
  const isRendering = task?.status === "running";

  return (
    <DebugSourceOverlay enabled={showDebugSources} source="src/startup/ScenePreviewWorkspace.tsx" contentClassName="debug-source-fill">
      <section className="panel preview-workspace">
      <PreviewCanvas preview={preview} isRendering={Boolean(isRendering)} playing={state.previewPlaying} />

      {isRendering ? (
        <div className="preview-progress">
          <span style={{ width: `${Math.round((task?.progress ?? 0) * 100)}%` }} />
        </div>
      ) : null}

      </section>
    </DebugSourceOverlay>
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
        <span>Select a scene or press Refresh.</span>
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
