import type {
  EditorFrameDto,
  EditorModeSessionDto,
  EditorSceneSnapshotQualityDto,
} from "../../../api/dto";
import type { SceneEditorPreviewSyncState } from "./sceneEditorPreviewSync";
import type {
  SceneEditorMode,
  SceneEditorPoint,
  SceneEditorResolution,
} from "./sceneEditorTypes";
import { zoomPercent } from "./sceneEditorTransforms";

export function SceneEditorHud({
  mode,
  mouseScenePoint,
  resolution,
  selectedEntityName,
  layoutSource,
  quality,
  previewSync,
  zoom,
  editorModeSession,
  frame,
}: {
  mode: SceneEditorMode;
  zoom: number;
  resolution: SceneEditorResolution;
  selectedEntityName?: string | null;
  mouseScenePoint?: SceneEditorPoint | null;
  layoutSource?: string;
  quality?: EditorSceneSnapshotQualityDto;
  previewSync?: SceneEditorPreviewSyncState;
  editorModeSession?: EditorModeSessionDto | null;
  frame?: EditorFrameDto | null;
}) {
  return (
    <>
      <div className="scene-editor-hud scene-editor-hud-top-left">
        <span>{resolution.width}x{resolution.height}</span>
        <span>{zoomPercent(zoom)}</span>
        <span>{mode.toUpperCase()}</span>
        {layoutSource ? <span>{layoutSource}</span> : null}
        {quality ? <span>editable {quality.editableObjects}/{quality.indexedEntities}</span> : null}
        {editorModeSession ? <span>session rev {editorModeSession.revision}</span> : <span>no editor session</span>}
        {frame ? <span>transport {frame.transport}</span> : null}
        {previewSync ? (
          <span className={`scene-editor-hud-sync scene-editor-hud-sync-${previewSync.status}`}>
            preview {previewSync.status}
          </span>
        ) : null}
      </div>
      <div className="scene-editor-hud scene-editor-hud-top-right">
        <span>{selectedEntityName ? `Selected: ${selectedEntityName}` : "No selection"}</span>
      </div>
      <div className="scene-editor-hud scene-editor-hud-bottom-left">
        {mouseScenePoint ? (
          <span>
            x: {Math.round(mouseScenePoint.x)} y: {Math.round(mouseScenePoint.y)}
          </span>
        ) : (
          <span>x: - y: -</span>
        )}
      </div>
    </>
  );
}
