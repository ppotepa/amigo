import { Expand, MousePointer2, Play } from "lucide-react";
import type { SceneEditorMode } from "./sceneEditorTypes";

export function SceneEditorModeDock({
  mode,
  onModeChange,
}: {
  mode: SceneEditorMode;
  onModeChange: (mode: SceneEditorMode) => void;
}) {
  return (
    <div
      className="scene-editor-floating-dock scene-editor-interaction-mode-dock"
      aria-label="Editor interaction mode"
      data-editor-chrome="true"
      onPointerDown={(event) => event.stopPropagation()}
      onPointerMove={(event) => event.stopPropagation()}
      onPointerUp={(event) => event.stopPropagation()}
      onClick={(event) => event.stopPropagation()}
    >
      <button
        className={`scene-editor-floating-button ${mode === "edit" ? "selected" : ""}`}
        type="button"
        title="Edit mode"
        onClick={() => onModeChange("edit")}
      >
        <Expand size={15} />
      </button>
      <button
        className={`scene-editor-floating-button ${mode === "preview" ? "selected" : ""}`}
        type="button"
        title="Preview mode"
        onClick={() => onModeChange("preview")}
      >
        <MousePointer2 size={15} />
      </button>
      <button
        className={`scene-editor-floating-button ${mode === "play" ? "selected" : ""}`}
        type="button"
        title="Play mode"
        onClick={() => onModeChange("play")}
      >
        <Play size={15} />
      </button>
    </div>
  );
}
