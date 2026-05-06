import { RotateCcw, RotateCw, Save, XCircle } from "lucide-react";
import type { EditorModeSessionDto } from "../../../api/dto";
import { DebugSourceLabel } from "../../../debug/debugSource";
import type { SceneEditorCanvasKind } from "./sceneEditorTypes";

export function SceneEditorToolbar({
  engineKind,
  engineLabel,
  editorModeSession,
  onDiscard,
  onRedo,
  onSave,
  onUndo,
}: {
  engineKind?: SceneEditorCanvasKind;
  engineLabel?: string;
  editorModeSession?: EditorModeSessionDto | null;
  onDiscard?: () => void;
  onRedo?: () => void;
  onSave?: () => void;
  onUndo?: () => void;
}) {
  const dirty = editorModeSession?.dirty ?? false;
  const canUndo = editorModeSession?.canUndo ?? false;
  const canRedo = editorModeSession?.canRedo ?? false;

  return (
    <div className="scene-editor-toolbar">
      <DebugSourceLabel source="src/features/scenes/editor/SceneEditorToolbar.tsx" />
      {engineLabel ? (
        <>
          <div className={`scene-editor-engine-pill scene-editor-engine-${engineKind ?? "2d"}`}>
            {engineLabel}
          </div>
          <div className="scene-editor-toolbar-separator" />
        </>
      ) : null}
      {editorModeSession ? (
        <>
          <button className="button button-tool" type="button" title="Undo" disabled={!canUndo} onClick={onUndo}>
            <RotateCcw size={14} />
          </button>
          <button className="button button-tool" type="button" title="Redo" disabled={!canRedo} onClick={onRedo}>
            <RotateCw size={14} />
          </button>
          <button className="button button-tool" type="button" title="Save editor changes" disabled={!dirty} onClick={onSave}>
            <Save size={14} />
          </button>
          <button className="button button-tool" type="button" title="Discard editor changes" disabled={!dirty} onClick={onDiscard}>
            <XCircle size={14} />
          </button>
        </>
      ) : null}
      <div className="scene-editor-toolbar-spacer" />
    </div>
  );
}
