import { useRef } from "react";
import { DebugSourceLabel } from "../../../debug/debugSource";
import { SceneEditorArtboard } from "./SceneEditorArtboard";
import { SceneEditorModeDock } from "./SceneEditorModeDock";
import { SceneEditorStatusBar } from "./SceneEditorStatusBar";
import { SceneEditorToolDock } from "./SceneEditorToolDock";
import { SceneEditorViewDock } from "./SceneEditorViewDock";
import type { SceneEditorCanvasProps } from "./sceneEditorTypes";
import { useSceneEditorPointerEvents } from "./useSceneEditorPointerEvents";
import { useSceneEditorViewportResize } from "./useSceneEditorViewportResize";

export function SceneEditorCanvas({
  mode,
  model,
  canvasKind,
  frame,
  previewSync,
  editorModeSession,
  selectedEntityId,
  tool,
  viewport,
  onFitViewport,
  onModeChange,
  onPointerEvent,
  onResetZoom,
  onToolChange,
  onViewportChange,
  onViewportResize,
  onZoomChange,
  onZoomIn,
  onZoomOut,
}: SceneEditorCanvasProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const selectedEntity = model.entities.find((entity) => entity.id === selectedEntityId) ?? null;
  const hasRealLayout = model.layoutSource === "runtime" || model.layoutSource === "document";
  const previewRegenerating = previewSync?.sceneId === model.sceneId && previewSync.status === "regenerating";
  const previewFailed = previewSync?.sceneId === model.sceneId && previewSync.status === "failed";

  useSceneEditorViewportResize({
    containerRef,
    editorModeSession,
    onViewportChange,
    onViewportResize,
    resolution: model.resolution,
    sceneId: model.sceneId,
  });

  const {
    engineHover,
    handlePointerCancel,
    handlePointerDown,
    handlePointerEnter,
    handlePointerLeave,
    handlePointerMove,
    handlePointerUp,
    handleWheel,
    mouseScenePoint,
  } = useSceneEditorPointerEvents({
    model,
    onPointerEvent,
    onViewportChange,
    viewport,
  });

  return (
    <div
      ref={containerRef}
      className={`scene-editor-canvas scene-editor-canvas-${tool} ${engineHover ? "is-engine-hover" : ""}`}
      onPointerDown={handlePointerDown}
      onPointerEnter={handlePointerEnter}
      onPointerLeave={handlePointerLeave}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerCancel}
      onWheel={handleWheel}
    >
      <DebugSourceLabel source="src/features/scenes/editor/SceneEditorCanvas.tsx" />
      <div className="scene-editor-grid" />
      <SceneEditorToolDock activeTool={tool} onToolChange={onToolChange} />
      <SceneEditorModeDock mode={mode} onModeChange={onModeChange} />
      <SceneEditorViewDock
        zoom={viewport.zoom}
        onFitViewport={onFitViewport}
        onResetZoom={onResetZoom}
        onZoomChange={onZoomChange}
        onZoomIn={onZoomIn}
        onZoomOut={onZoomOut}
      />
      <SceneEditorArtboard frame={frame} resolution={model.resolution} viewport={viewport} />
      {!hasRealLayout ? (
        <div className="scene-editor-layout-warning">
          <strong>Editor layout unavailable</strong>
          <span>
            Real transform/bounds snapshot is not available yet. Picking and manipulation must come
            from the engine session.
          </span>
        </div>
      ) : null}
      {previewRegenerating ? (
        <div className="scene-editor-preview-sync scene-editor-preview-sync-updating">
          <strong>Preview updating...</strong>
          <span>The document snapshot changed. Waiting for the engine viewport frame to catch up.</span>
        </div>
      ) : null}
      {previewFailed ? (
        <div className="scene-editor-preview-sync scene-editor-preview-sync-failed">
          <strong>Preview regeneration failed</strong>
          <span>{previewSync?.message ?? "The snapshot updated, but preview rendering failed."}</span>
        </div>
      ) : null}
      <SceneEditorStatusBar
        canvasKind={canvasKind}
        mode={mode}
        mouseScenePoint={mouseScenePoint}
        selectedEntityName={selectedEntity?.name ?? null}
        quality={model.quality}
        previewSync={previewSync?.sceneId === model.sceneId ? previewSync : undefined}
        zoom={viewport.zoom}
        session={editorModeSession}
        frame={frame}
        tool={tool}
      />
    </div>
  );
}
