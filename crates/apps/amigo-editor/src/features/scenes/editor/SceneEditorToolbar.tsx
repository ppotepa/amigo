import type { EditorModeSessionDto } from "../../../api/dto";
import type { SceneEditorCanvasKind } from "./sceneEditorTypes";

export function SceneEditorToolbar({
  engineKind,
  engineLabel,
  editorModeSession,
}: {
  engineKind?: SceneEditorCanvasKind;
  engineLabel?: string;
  editorModeSession?: EditorModeSessionDto | null;
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
      {editorModeSession ? (
        <div className="scene-editor-session-pill">
          Engine editor · rev {editorModeSession.revision}
          {editorModeSession.dirty ? " · dirty" : ""}
        </div>
      ) : null}
    </div>
  );
}
