import { Gauge, Play } from "lucide-react";
import type { EditorSceneSummaryDto, ScenePreviewDto } from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { EngineSlideshowPreview } from "../../startup/EngineSlideshowPreview";

export function ScenePreviewWorkbench({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <ScenePreviewWorkbenchView
      detailsLoaded={Boolean(services.details)}
      playing={services.previewPlaying ?? true}
      preview={services.preview}
      previewTask={services.previewTask}
      selectedScene={services.selectedScene ?? null}
    />
  );
}

function ScenePreviewWorkbenchView({
  playing,
  preview,
  previewTask,
  selectedScene,
}: {
  detailsLoaded: boolean;
  playing: boolean;
  preview?: ScenePreviewDto;
  previewTask?: { progress?: number; status: string };
  selectedScene: EditorSceneSummaryDto | null;
}) {
  return (
    <div className="scene-workbench">
      <div className="scene-workbench-toolbar">
        <div className="scene-heading">
          <span className="dock-icon dock-icon-cyan"><Play size={14} /></span>
          <strong>{selectedScene?.label ?? "Scene Preview"}</strong>
          <span>{selectedScene?.documentPath ?? "No scene selected"}</span>
          <span className="badge badge-info">engine preview</span>
        </div>
        <div className="scene-heading-actions">
          <button className="button button-tool" type="button">Fit</button>
          <button className="button button-tool" type="button">1:1</button>
        </div>
      </div>

      <div className="main-preview-stage">
        {previewTask?.status === "running" ? (
          <div className="preview-canvas preview-loading">
            <div className="spinner" />
            <strong>Rendering preview...</strong>
            <span>{Math.round((previewTask.progress ?? 0) * 100)}%</span>
          </div>
        ) : preview?.status === "ready" && preview.frameUrls.length > 0 ? (
          <EngineSlideshowPreview preview={preview} playing={playing} />
        ) : (
          <div className="preview-canvas preview-empty">
            <Gauge size={38} />
            <strong>No workspace preview yet</strong>
            <span>Select a scene or regenerate preview.</span>
          </div>
        )}
      </div>
    </div>
  );
}
