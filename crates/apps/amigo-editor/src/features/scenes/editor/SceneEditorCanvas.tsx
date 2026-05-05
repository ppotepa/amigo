import { useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorLiveCommandResultDto,
  EditorLiveSceneSessionDto,
  EditorTransform2Dto,
  ScenePreviewDto,
} from "../../../api/dto";
import {
  Expand,
  FileText,
  Hand,
  Maximize2,
  Minus,
  MousePointer2,
  Move,
  Play,
  Plus,
  RadioTower,
  RotateCw,
  Scaling,
} from "lucide-react";
import type { SceneEditorModeKind } from "./sceneEditorMode";
import type { SceneEditorPreviewSyncState } from "./sceneEditorPreviewSync";
import type {
  SceneEditorDragState,
  SceneEditorEntity,
  SceneEditorMode,
  SceneEditorModel,
  SceneEditorPoint,
  SceneEditorTool,
  SceneEditorTransform,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import {
  clampZoom,
  fitResolutionToViewport,
  screenToScene,
  translateViewport,
  zoomAroundScreenPoint,
} from "./sceneEditorTransforms";
import { applyDraftTransform } from "./sceneEditorModel";
import { SceneEditorArtboard } from "./SceneEditorArtboard";
import { SceneEditorHud } from "./SceneEditorHud";
import { emitSceneEditorCommand } from "./sceneEditorCommands";

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

export function SceneEditorCanvas({
  editorModeKind,
  liveAvailable,
  liveOpening,
  liveSession,
  mode,
  model,
  onApplyCommand,
  onApplyLiveTransform,
  onEditorModeKindChange,
  onFitViewport,
  onModeChange,
  onSelectEntity,
  onResetZoom,
  onToolChange,
  onViewportChange,
  onZoomIn,
  onZoomOut,
  preview,
  previewSync,
  selectedEntityId,
  tool,
  viewport,
}: {
  model: SceneEditorModel;
  preview?: ScenePreviewDto;
  previewSync?: SceneEditorPreviewSyncState;
  selectedEntityId: string | null;
  editorModeKind: SceneEditorModeKind;
  liveAvailable: boolean;
  liveOpening?: boolean;
  liveSession?: EditorLiveSceneSessionDto | null;
  mode: SceneEditorMode;
  tool: SceneEditorTool;
  viewport: SceneEditorViewportState;
  onViewportChange: (viewport: SceneEditorViewportState) => void;
  onEditorModeKindChange: (mode: SceneEditorModeKind) => void;
  onModeChange: (mode: SceneEditorMode) => void;
  onToolChange: (tool: SceneEditorTool) => void;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onSelectEntity: (entityId: string | null) => void;
  onApplyCommand?: (command: EditorCommandDto) => Promise<EditorCommandResultDto | null>;
  onApplyLiveTransform?: (entityId: string, transform: EditorTransform2Dto) => Promise<EditorLiveCommandResultDto | null>;
}) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<SceneEditorDragState | null>(null);
  const panRef = useRef<{ pointerId: number; start: SceneEditorPoint; viewport: SceneEditorViewportState } | null>(null);
  const [mouseScenePoint, setMouseScenePoint] = useState<SceneEditorPoint | null>(null);
  const [draftTransforms, setDraftTransforms] = useState<Record<string, Partial<SceneEditorTransform>>>({});
  const [committingEntityId, setCommittingEntityId] = useState<string | null>(null);
  const liveTransformThrottleRef = useRef<{
    entityId: string;
    transform: SceneEditorTransform;
    timeout: number | null;
  } | null>(null);

  const entities = useMemo(() => {
    return model.entities.map((entity) => applyDraftTransform(entity, draftTransforms[entity.id]));
  }, [draftTransforms, model.entities]);

  const selectedEntity = entities.find((entity) => entity.id === selectedEntityId) ?? null;
  const hasRealLayout = model.layoutSource === "runtime" || model.layoutSource === "document";
  const isLiveMode = editorModeKind === "live" && Boolean(liveSession);
  const previewRegenerating = !isLiveMode && previewSync?.sceneId === model.sceneId && previewSync.status === "regenerating";
  const previewFailed = !isLiveMode && previewSync?.sceneId === model.sceneId && previewSync.status === "failed";
  const canEditObjects = mode === "edit" && hasRealLayout && !committingEntityId && !previewRegenerating;

  useEffect(() => {
    const element = containerRef.current;
    if (!element) return;
    const rect = element.getBoundingClientRect();
    onViewportChange(fitResolutionToViewport(model.resolution, {
      width: rect.width,
      height: rect.height,
    }));
  }, [model.sceneId, model.resolution.width, model.resolution.height, onViewportChange]);

  function localPoint(event: { currentTarget: Element; clientX: number; clientY: number }): SceneEditorPoint {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };
  }

  function updateMousePoint(event: React.PointerEvent<HTMLDivElement>) {
    setMouseScenePoint(screenToScene(localPoint(event), viewport));
  }

  function beginEntityDrag(
    event: React.PointerEvent<HTMLButtonElement>,
    entity: SceneEditorEntity,
  ) {
    if (committingEntityId) return;
    if (!canEditObjects) return;
    if (entity.locked) return;

    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    onSelectEntity(entity.id);
    emitSceneEditorCommand({ type: "selectEntity", entityId: entity.id });
    dragRef.current = {
      entityId: entity.id,
      pointerId: event.pointerId,
      startScreen: {
        x: event.clientX,
        y: event.clientY,
      },
      startTransform: entity.transform,
    };
  }

  function liveTransformFromSceneTransform(transform: SceneEditorTransform): EditorTransform2Dto {
    return {
      x: transform.x,
      y: transform.y,
      rotation: transform.rotation,
      scaleX: transform.scaleX,
      scaleY: transform.scaleY,
    };
  }

  function queueLiveTransform(entityId: string, transform: SceneEditorTransform) {
    if (!isLiveMode || !onApplyLiveTransform) return;

    const pending = liveTransformThrottleRef.current;
    if (pending?.timeout) {
      window.clearTimeout(pending.timeout);
    }

    liveTransformThrottleRef.current = {
      entityId,
      transform,
      timeout: window.setTimeout(() => {
        const next = liveTransformThrottleRef.current;
        if (!next) return;
        liveTransformThrottleRef.current = null;
        void onApplyLiveTransform(next.entityId, liveTransformFromSceneTransform(next.transform));
      }, 33),
    };
  }

  function flushLiveTransform(entityId: string, transform: SceneEditorTransform) {
    const pending = liveTransformThrottleRef.current;
    if (pending?.timeout) {
      window.clearTimeout(pending.timeout);
    }
    liveTransformThrottleRef.current = null;
    void onApplyLiveTransform?.(entityId, liveTransformFromSceneTransform(transform));
  }

  function onPointerMove(event: React.PointerEvent<HTMLDivElement>) {
    updateMousePoint(event);
    const drag = dragRef.current;
    if (drag) {
      const delta = {
        x: (event.clientX - drag.startScreen.x) / viewport.zoom,
        y: (event.clientY - drag.startScreen.y) / viewport.zoom,
      };
      const transform = {
        ...drag.startTransform,
        x: Math.round(drag.startTransform.x + delta.x),
        y: Math.round(drag.startTransform.y + delta.y),
      };
      setDraftTransforms((current) => ({
        ...current,
        [drag.entityId]: transform,
      }));
      queueLiveTransform(drag.entityId, transform);
      return;
    }
    const pan = panRef.current;
    if (pan) {
      onViewportChange(translateViewport(pan.viewport, {
        x: event.clientX - pan.start.x,
        y: event.clientY - pan.start.y,
      }));
    }
  }

  function onPointerUp(event: React.PointerEvent<HTMLDivElement>) {
    const drag = dragRef.current;
    if (drag) {
      const draft = draftTransforms[drag.entityId];
      if (draft?.x != null && draft?.y != null) {
        const entity = entities.find((candidate) => candidate.id === drag.entityId);
        const transform = {
          ...(entity?.transform ?? drag.startTransform),
          ...draft,
        };
        emitSceneEditorCommand({
          type: "moveEntity",
          entityId: drag.entityId,
          x: draft.x,
          y: draft.y,
        });
        if (isLiveMode) {
          flushLiveTransform(drag.entityId, transform);
          setDraftTransforms((current) => {
            const next = { ...current };
            delete next[drag.entityId];
            return next;
          });
          dragRef.current = null;
          return;
        }
        setCommittingEntityId(drag.entityId);
        void onApplyCommand?.({
          type: "SetEntityTransform2D",
          sceneId: model.sceneId,
          entityId: drag.entityId,
          transform,
        }).then((result) => {
          if (result?.ok) {
            setDraftTransforms((current) => {
              const next = { ...current };
              delete next[drag.entityId];
              return next;
            });
            return;
          }
          if (result && !result.ok) {
            setDraftTransforms((current) => {
              const next = { ...current };
              delete next[drag.entityId];
              return next;
            });
            console.warn("[SceneEditor] Transform command rejected:", result.message);
          }
        }).finally(() => {
          setCommittingEntityId((current) =>
            current === drag.entityId ? null : current
          );
        });
      }
      dragRef.current = null;
    }
    if (panRef.current?.pointerId === event.pointerId) {
      panRef.current = null;
    }
  }

  function onBackgroundPointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (!canEditObjects) return;
    if (tool !== "pan") return;
    event.currentTarget.setPointerCapture(event.pointerId);
    panRef.current = {
      pointerId: event.pointerId,
      start: {
        x: event.clientX,
        y: event.clientY,
      },
      viewport,
    };
  }

  function onWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (!event.ctrlKey && !event.metaKey) return;
    event.preventDefault();
    const point = localPoint(event);
    const factor = event.deltaY > 0 ? 0.9 : 1.1;
    onViewportChange(zoomAroundScreenPoint({
      viewport,
      point,
      nextZoom: clampZoom(viewport.zoom * factor),
    }));
  }

  return (
    <div
      ref={containerRef}
      className={`scene-editor-canvas scene-editor-canvas-${tool} scene-editor-canvas-mode-${editorModeKind}`}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerUp}
      onPointerDown={onBackgroundPointerDown}
      onWheel={onWheel}
    >
      <div className="scene-editor-grid" />
      <SceneEditorToolDock
        activeTool={tool}
        onToolChange={onToolChange}
      />
      <SceneEditorSourceModeDock
        editorModeKind={editorModeKind}
        liveAvailable={liveAvailable}
        liveOpening={liveOpening}
        onEditorModeKindChange={onEditorModeKindChange}
      />
      <SceneEditorInteractionModeDock
        mode={mode}
        onModeChange={onModeChange}
      />
      <SceneEditorViewDock
        zoom={viewport.zoom}
        onFitViewport={onFitViewport}
        onResetZoom={onResetZoom}
        onZoomIn={onZoomIn}
        onZoomOut={onZoomOut}
      />
      <SceneEditorArtboard
        entities={hasRealLayout ? entities : []}
        mode={canEditObjects ? mode : "preview"}
        preview={preview}
        resolution={model.resolution}
        selectedEntityId={selectedEntityId}
        tool={tool}
        viewport={viewport}
        onSelectEntity={onSelectEntity}
        onBeginEntityDrag={beginEntityDrag}
      />
      {!hasRealLayout ? (
        <div className="scene-editor-layout-warning">
          <strong>Editor layout unavailable</strong>
          <span>
            Real transform/bounds snapshot is not available yet. Entity selection from the right
            context panel still works, but viewport picking and dragging are disabled.
          </span>
        </div>
      ) : null}
      {committingEntityId ? (
        <div className="scene-editor-preview-sync scene-editor-preview-sync-pending">
          <strong>Saving transform...</strong>
          <span>Committing scene YAML and waiting for the authoritative snapshot.</span>
        </div>
      ) : null}
      {previewRegenerating ? (
        <div className="scene-editor-preview-sync scene-editor-preview-sync-updating">
          <strong>Preview updating...</strong>
          <span>
            Bounds already use the latest snapshot. The rendered frame is being regenerated.
          </span>
        </div>
      ) : null}
      {previewFailed ? (
        <div className="scene-editor-preview-sync scene-editor-preview-sync-failed">
          <strong>Preview regeneration failed</strong>
          <span>{previewSync?.message ?? "The scene snapshot was updated, but preview rendering failed."}</span>
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
        editorModeKind={editorModeKind}
        liveSession={liveSession}
        zoom={viewport.zoom}
      />
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

function SceneEditorSourceModeDock({
  editorModeKind,
  liveAvailable,
  liveOpening,
  onEditorModeKindChange,
}: {
  editorModeKind: SceneEditorModeKind;
  liveAvailable: boolean;
  liveOpening?: boolean;
  onEditorModeKindChange: (mode: SceneEditorModeKind) => void;
}) {
  return (
    <div className="scene-editor-floating-dock scene-editor-source-mode-dock" aria-label="Document or Live mode">
      <button
        className={`scene-editor-floating-button ${editorModeKind === "document" ? "selected" : ""}`}
        type="button"
        title="Document Mode: commit transforms to scene.yml and regenerate preview."
        onClick={() => onEditorModeKindChange("document")}
      >
        <FileText size={15} />
      </button>
      <button
        className={`scene-editor-floating-button ${editorModeKind === "live" ? "selected" : ""}`}
        type="button"
        title={
          liveAvailable
            ? "Live Mode: edit an in-memory editor snapshot and save/discard later."
            : "Live Mode is not available for this canvas."
        }
        disabled={!liveAvailable || liveOpening}
        onClick={() => onEditorModeKindChange("live")}
      >
        <RadioTower size={15} />
        {liveOpening ? <span className="scene-editor-floating-dot" aria-hidden="true" /> : null}
      </button>
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
  onZoomIn,
  onZoomOut,
}: {
  zoom: number;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
}) {
  return (
    <div className="scene-editor-floating-dock scene-editor-view-dock" aria-label="Viewport controls">
      <button className="scene-editor-floating-button" type="button" title="Zoom in" onClick={onZoomIn}>
        <Plus size={15} />
      </button>
      <button className="scene-editor-floating-zoom" type="button" title="Reset zoom" onClick={onResetZoom}>
        {Math.round(zoom * 100)}%
      </button>
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
