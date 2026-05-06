import { useCallback, useEffect, useRef, useState } from "react";
import type React from "react";
import {
  Expand,
  Hand,
  Maximize2,
  Minus,
  MousePointer2,
  Move,
  Play,
  Plus,
  RotateCw,
  Scaling,
} from "lucide-react";
import type { EditorPointerEventDto, EditorViewportDto } from "../../../api/dto";
import type {
  SceneEditorMode,
  SceneEditorPoint,
  SceneEditorTool,
} from "./sceneEditorTypes";
import { clampZoom, fitResolutionToViewport, screenToScene, zoomAroundScreenPoint } from "./sceneEditorTransforms";
import { SceneEditorArtboard } from "./SceneEditorArtboard";
import { SceneEditorGeometryDebugPanel } from "./SceneEditorGeometryDebugPanel";
import { SceneEditorHud } from "./SceneEditorHud";
import type { SceneEditorCanvasProps } from "./sceneEditorTypes";
import { SceneCanvasKindBadge } from "./canvas/SceneCanvasKindBadge";

const TOOL_DOCK: Array<{
  id: SceneEditorTool;
  label: string;
  icon: React.ReactNode;
}> = [
  { id: "select", label: "Select", icon: <MousePointer2 size={15} /> },
  { id: "move", label: "Move", icon: <Move size={15} /> },
  { id: "scale", label: "Scale", icon: <Scaling size={15} /> },
  { id: "rotate", label: "Rotate", icon: <RotateCw size={15} /> },
  { id: "pan", label: "Pan", icon: <Hand size={15} /> },
];

function sameEditorViewport(a: EditorViewportDto, b: EditorViewportDto): boolean {
  return (
    Math.abs(a.cssWidth - b.cssWidth) < 0.5 &&
    Math.abs(a.cssHeight - b.cssHeight) < 0.5 &&
    a.renderWidth === b.renderWidth &&
    a.renderHeight === b.renderHeight &&
    Math.abs(a.devicePixelRatio - b.devicePixelRatio) < 0.001
  );
}

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
  const lastEditorViewportRef = useRef<EditorViewportDto | null>(null);
  const [mouseScenePoint, setMouseScenePoint] = useState<SceneEditorPoint | null>(null);
  const selectedEntity = model.entities.find((entity) => entity.id === selectedEntityId) ?? null;
  const hasRealLayout = model.layoutSource === "runtime" || model.layoutSource === "document";
  const previewRegenerating = previewSync?.sceneId === model.sceneId && previewSync.status === "regenerating";
  const previewFailed = previewSync?.sceneId === model.sceneId && previewSync.status === "failed";

  useEffect(() => {
    const element = containerRef.current;
    if (!element) return;
    const rect = element.getBoundingClientRect();
    onViewportChange(fitResolutionToViewport(model.resolution, {
      width: rect.width,
      height: rect.height,
    }));
  }, [model.sceneId, model.resolution.height, model.resolution.width, onViewportChange]);

  const buildEditorViewport = useCallback((element: HTMLElement) => {
    const rect = element.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    return {
      cssWidth: rect.width,
      cssHeight: rect.height,
      renderWidth: Math.max(1, Math.round(rect.width * dpr)),
      renderHeight: Math.max(1, Math.round(rect.height * dpr)),
      devicePixelRatio: dpr,
    };
  }, []);

  const emitViewportResize = useCallback((element: HTMLElement) => {
    if (!onViewportResize) return;
    const nextViewport = buildEditorViewport(element);
    const previousViewport = lastEditorViewportRef.current;
    if (previousViewport && sameEditorViewport(previousViewport, nextViewport)) {
      return;
    }
    lastEditorViewportRef.current = nextViewport;
    void onViewportResize(nextViewport);
  }, [buildEditorViewport, onViewportResize]);

  useEffect(() => {
    const element = containerRef.current;
    if (!element || !onViewportResize || !editorModeSession) return;

    let raf = 0;
    lastEditorViewportRef.current = null;
    const resizeObserver = new ResizeObserver(() => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        emitViewportResize(element);
      });
    });

    resizeObserver.observe(element);
    emitViewportResize(element);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver.disconnect();
    };
  }, [editorModeSession?.editorModeSessionId, emitViewportResize, onViewportResize]);

  function localPoint(event: { currentTarget: Element; clientX: number; clientY: number }): SceneEditorPoint {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };
  }

  function updateMousePoint(event: React.PointerEvent<HTMLDivElement>) {
    setMouseScenePoint(screenToScene(localPoint(event), viewport, model.resolution));
  }

  function toEditorPointerEvent(
    event: React.PointerEvent<HTMLDivElement>,
    type: EditorPointerEventDto["type"],
  ): EditorPointerEventDto {
    const rect = event.currentTarget.getBoundingClientRect();
    const viewport = buildEditorViewport(event.currentTarget);

    return {
      type,
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
      button: event.button,
      buttons: event.buttons,
      pointerId: event.pointerId,
      modifiers: {
        shift: event.shiftKey,
        ctrl: event.ctrlKey,
        alt: event.altKey,
        meta: event.metaKey,
      },
      viewport,
    };
  }

  async function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    updateMousePoint(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerDown"));
  }

  async function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    updateMousePoint(event);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerMove"));
  }

  async function handlePointerUp(event: React.PointerEvent<HTMLDivElement>) {
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerUp"));
  }

  async function handlePointerCancel(event: React.PointerEvent<HTMLDivElement>) {
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerCancel"));
  }

  async function handleWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      const point = localPoint(event);
      const factor = event.deltaY > 0 ? 0.9 : 1.1;
      onViewportChange(zoomAroundScreenPoint({
        viewport,
        point,
        nextZoom: clampZoom(viewport.zoom * factor),
      }));
    }
  }

  return (
    <div
      ref={containerRef}
      className={`scene-editor-canvas scene-editor-canvas-${tool}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerCancel}
      onWheel={handleWheel}
    >
      <div className="scene-editor-grid" />
      <SceneEditorToolDock activeTool={tool} onToolChange={onToolChange} />
      <SceneEditorInteractionModeDock mode={mode} onModeChange={onModeChange} />
      <SceneEditorViewDock
        zoom={viewport.zoom}
        onFitViewport={onFitViewport}
        onResetZoom={onResetZoom}
        onZoomChange={onZoomChange}
        onZoomIn={onZoomIn}
        onZoomOut={onZoomOut}
      />
      <SceneCanvasKindBadge kind={canvasKind} />
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
      <SceneEditorHud
        mode={mode}
        mouseScenePoint={mouseScenePoint}
        resolution={model.resolution}
        selectedEntityName={selectedEntity?.name ?? null}
        layoutSource={model.layoutSource}
        quality={model.quality}
        previewSync={previewSync?.sceneId === model.sceneId ? previewSync : undefined}
        zoom={viewport.zoom}
        editorModeSession={editorModeSession}
        frame={frame}
      />
      <SceneEditorGeometryDebugPanel model={model} selectedEntity={selectedEntity} />
      {editorModeSession ? (
        <div className="scene-editor-session-overlay">
          Engine session rev {editorModeSession.revision}
          {editorModeSession.dirty ? " · dirty" : ""}
        </div>
      ) : null}
    </div>
  );
}

function SceneEditorToolDock({
  activeTool,
  onToolChange,
}: {
  activeTool: SceneEditorTool;
  onToolChange: (tool: SceneEditorTool) => void;
}) {
  return (
    <div className="scene-editor-floating-dock scene-editor-tool-dock" aria-label="Transform tools">
      {TOOL_DOCK.map((entry) => (
        <button
          key={entry.id}
          className={`scene-editor-floating-button ${activeTool === entry.id ? "selected" : ""}`}
          type="button"
          title={entry.label}
          onClick={() => onToolChange(entry.id)}
        >
          {entry.icon}
        </button>
      ))}
    </div>
  );
}

function SceneEditorInteractionModeDock({
  mode,
  onModeChange,
}: {
  mode: SceneEditorMode;
  onModeChange: (mode: SceneEditorMode) => void;
}) {
  return (
    <div className="scene-editor-floating-dock scene-editor-interaction-mode-dock" aria-label="Editor interaction mode">
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

function SceneEditorViewDock({
  zoom,
  onFitViewport,
  onResetZoom,
  onZoomChange,
  onZoomIn,
  onZoomOut,
}: {
  zoom: number;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomChange: (zoom: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
}) {
  const zoomMin = 0.25;
  const zoomMax = 4;
  const normalizedZoom = Math.max(0, Math.min(1, (zoom - zoomMin) / (zoomMax - zoomMin)));

  function zoomFromPointer(event: React.PointerEvent<HTMLButtonElement>) {
    const rect = event.currentTarget.getBoundingClientRect();
    const ratio = 1 - ((event.clientY - rect.top) / rect.height);
    const nextZoom = zoomMin + Math.max(0, Math.min(1, ratio)) * (zoomMax - zoomMin);
    onZoomChange(clampZoom(nextZoom));
  }

  return (
    <div className="scene-editor-floating-dock scene-editor-view-dock" aria-label="Viewport controls">
      <button className="scene-editor-floating-button" type="button" title="Zoom in" onClick={onZoomIn}>
        <Plus size={15} />
      </button>
      <div className="scene-editor-zoom-slider" title={`Zoom ${Math.round(zoom * 100)}%`}>
        <span>{Math.round(zoom * 100)}%</span>
        <button
          aria-label="Zoom"
          className="scene-editor-zoom-slider-track"
          type="button"
          onDoubleClick={onResetZoom}
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            zoomFromPointer(event);
          }}
          onPointerMove={(event) => {
            if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
            zoomFromPointer(event);
          }}
        >
          <span className="scene-editor-zoom-slider-fill" style={{ height: `${normalizedZoom * 100}%` }} />
          <span className="scene-editor-zoom-slider-thumb" style={{ bottom: `${normalizedZoom * 100}%` }} />
        </button>
      </div>
      <button className="scene-editor-floating-button" type="button" title="Zoom out" onClick={onZoomOut}>
        <Minus size={15} />
      </button>
      <span className="scene-editor-floating-separator" />
      <button className="scene-editor-floating-button" type="button" title="Fit to view" onClick={onFitViewport}>
        <Maximize2 size={15} />
      </button>
    </div>
  );
}
