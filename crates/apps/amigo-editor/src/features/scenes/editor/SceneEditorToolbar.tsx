import {
  Check,
  RadioTower,
  RotateCcw,
  X,
} from "lucide-react";
import type { EditorLiveSceneSessionDto } from "../../../api/dto";
import type { SceneEditorModeKind } from "./sceneEditorMode";
import type {
  SceneEditorCanvasKind,
} from "./sceneEditorTypes";

export function SceneEditorToolbar({
  engineKind,
  engineLabel,
  editorModeKind,
  liveError,
  liveOpening,
  liveSession,
  onCloseLive,
  onCommitLive,
  onDiscardLive,
}: {
  engineKind?: SceneEditorCanvasKind;
  engineLabel?: string;
  editorModeKind: SceneEditorModeKind;
  liveOpening?: boolean;
  liveError?: string | null;
  liveSession?: EditorLiveSceneSessionDto | null;
  onCommitLive?: () => void;
  onDiscardLive?: () => void;
  onCloseLive?: () => void;
}) {
  return (
    <div className="scene-editor-toolbar">
      {engineLabel ? (
        <>
          <div className={`scene-editor-engine-pill scene-editor-engine-${engineKind ?? "2d"}`}>
            {engineLabel}
          </div>
          <div className="scene-editor-toolbar-separator" />
        </>
      ) : null}
      <div className="scene-editor-toolbar-spacer" />
      <SceneEditorLiveToolbarStatus
        error={liveError}
        mode={editorModeKind}
        opening={liveOpening}
        session={liveSession}
        onClose={onCloseLive}
        onCommit={onCommitLive}
        onDiscard={onDiscardLive}
      />
    </div>
  );
}

function SceneEditorLiveToolbarStatus({
  error,
  mode,
  opening,
  session,
  onClose,
  onCommit,
  onDiscard,
}: {
  mode: SceneEditorModeKind;
  opening?: boolean;
  error?: string | null;
  session?: EditorLiveSceneSessionDto | null;
  onCommit?: () => void;
  onDiscard?: () => void;
  onClose?: () => void;
}) {
  if (mode !== "live") return null;

  if (opening) {
    return (
      <div className="scene-editor-live-compact scene-editor-live-compact-pending" title="Opening Live Mode">
        <RadioTower size={13} />
        <span>opening</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="scene-editor-live-compact scene-editor-live-compact-error" title={error}>
        <RadioTower size={13} />
        <span>failed</span>
      </div>
    );
  }

  if (!session) {
    return (
      <div className="scene-editor-live-compact scene-editor-live-compact-muted" title="Live Mode is selected, but no live session is open.">
        <RadioTower size={13} />
        <span>not open</span>
      </div>
    );
  }

  return (
    <div className="scene-editor-live-compact" title={`Live revision ${session.revision}${session.dirty ? " with unsaved changes" : " clean"}`}>
      <RadioTower size={13} />
      <span>r{session.revision}</span>
      <span className={session.dirty ? "scene-editor-live-dirty" : "scene-editor-live-clean"}>
        {session.dirty ? "dirty" : "clean"}
      </span>
      <button
        className="scene-editor-live-action"
        type="button"
        title="Save Live changes to YAML"
        disabled={!session.dirty}
        onClick={onCommit}
      >
        <Check size={13} />
      </button>
      <button
        className="scene-editor-live-action"
        type="button"
        title="Discard Live changes"
        disabled={!session.dirty}
        onClick={onDiscard}
      >
        <RotateCcw size={13} />
      </button>
      <button
        className="scene-editor-live-action"
        type="button"
        title="Close Live Mode"
        onClick={onClose}
      >
        <X size={13} />
      </button>
    </div>
  );
}
