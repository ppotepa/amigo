import { useCallback, useMemo, useRef, useState } from "react";
import { FileCode2, FileText } from "lucide-react";
import type { EditorComponentProps } from "../../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import { sceneYamlSource } from "../../files/yamlSourceRefs";
import { buildSceneEditorModel } from "./sceneEditorModel";
import type {
  SceneEditorMode,
  SceneEditorTool,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import type { SceneEditorModeKind } from "./sceneEditorMode";
import {
  selectSceneCanvasEngine,
  selectSceneCanvasKind,
} from "./canvas/selectSceneCanvasEngine";
import { SceneCanvasKindBadge } from "./canvas/SceneCanvasKindBadge";
import {
  clampZoom,
  fitResolutionToViewport,
} from "./sceneEditorTransforms";
import { SceneEditorToolbar } from "./SceneEditorToolbar";
import { emitSceneEditorCommand } from "./sceneEditorCommands";

export function SceneEditorWorkbench({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const selectedScene = services.selectedScene ?? null;
  const selectedEntityId = services.selectedEntity?.id ?? null;
  const [mode, setMode] = useState<SceneEditorMode>("edit");
  const [tool, setTool] = useState<SceneEditorTool>("select");
  const sceneEditorMode = services.sceneEditorMode ?? "document";
  const [viewport, setViewport] = useState<SceneEditorViewportState>({
    zoom: 1,
    panX: 0,
    panY: 0,
  });
  const shellRef = useRef<HTMLDivElement | null>(null);
  const model = useMemo(() => {
    if (!selectedScene) return null;
    return buildSceneEditorModel({
      sceneId: selectedScene.id,
      preview: services.preview,
      hierarchy: services.hierarchy,
      snapshot: services.editorSnapshot,
    });
  }, [selectedScene, services.preview, services.hierarchy, services.editorSnapshot]);

  const updateViewport = useCallback((next: SceneEditorViewportState) => {
    setViewport(next);
  }, []);
  const canvasEngine = useMemo(() => {
    const kind = selectSceneCanvasKind({
      scene: selectedScene,
      snapshot: services.editorSnapshot,
    });
    return selectSceneCanvasEngine(kind);
  }, [selectedScene, services.editorSnapshot]);
  const Canvas = canvasEngine.render;
  const liveAvailable = canvasEngine.kind === "2d" && Boolean(services.openEditorLiveSession);

  if (!selectedScene || !model) {
    return (
      <div className="scene-editor-empty">
        <p className="muted workspace-empty">Select a scene to open the editor.</p>
      </div>
    );
  }

  function fitViewport() {
    const rect = shellRef.current?.getBoundingClientRect();
    if (!rect || !model) return;
    setViewport(fitResolutionToViewport(model.resolution, {
      width: rect.width,
      height: rect.height,
    }));
  }

  function resetZoom() {
    setViewport((current) => ({
      ...current,
      zoom: 1,
    }));
  }

  function zoomIn() {
    setViewport((current) => ({
      ...current,
      zoom: clampZoom(current.zoom * 1.15),
    }));
  }

  function zoomOut() {
    setViewport((current) => ({
      ...current,
      zoom: clampZoom(current.zoom / 1.15),
    }));
  }

  function selectEntity(entityId: string | null) {
    if (entityId) {
      services.selectSceneEntity?.(entityId);
    }
    emitSceneEditorCommand({
      type: "selectEntity",
      entityId,
    });
  }

  function changeSceneEditorMode(next: SceneEditorModeKind) {
    if (next === "live") {
      services.setSceneEditorMode?.("live");
      void services.openEditorLiveSession?.();
      return;
    }
    services.setSceneEditorMode?.("document");
  }

  return (
    <div className="scene-editor-workbench">
      <header className="scene-editor-header">
        <div className="scene-editor-title">
          <strong>{selectedScene.label}</strong>
          <small>{selectedScene.documentPath}</small>
        </div>
        <div className="scene-editor-header-actions">
          <SceneEditorToolbar
            engineKind={canvasEngine.kind}
            engineLabel={canvasEngine.label}
            editorModeKind={sceneEditorMode}
            liveError={services.editorLiveError}
            liveOpening={services.editorLiveSessionOpening}
            liveSession={services.editorLiveSession ?? null}
            onCommitLive={() => void services.commitEditorLiveSession?.()}
            onDiscardLive={() => void services.discardEditorLiveSession?.()}
            onCloseLive={() => void services.closeEditorLiveSession?.()}
          />
          <button
            className="button button-tool"
            type="button"
            title="Open scene YAML"
            disabled={!services.showYamlView}
            onClick={() => {
              const source = sceneYamlSource(selectedScene);
              if (source) services.showYamlView?.(source);
            }}
          >
            <FileText size={14} />
          </button>
          <button
            className="button button-tool"
            type="button"
            title="Open scene script"
            disabled={!selectedScene.scriptPath || !services.openSceneScript}
            onClick={() => services.openSceneScript?.(selectedScene)}
          >
            <FileCode2 size={14} />
          </button>
        </div>
      </header>
      <div ref={shellRef} className="scene-editor-shell">
        <SceneCanvasKindBadge kind={canvasEngine.kind} />
        <Canvas
          scene={selectedScene}
          mode={mode}
          model={model}
          snapshot={services.editorSnapshot}
          preview={services.preview}
          previewSync={services.editorPreviewSync}
          selectedEntityId={selectedEntityId}
          editorModeKind={sceneEditorMode}
          liveAvailable={liveAvailable}
          liveOpening={services.editorLiveSessionOpening}
          liveSession={services.editorLiveSession ?? null}
          tool={tool}
          viewport={viewport}
          onEditorModeKindChange={changeSceneEditorMode}
          onModeChange={setMode}
          onToolChange={setTool}
          onFitViewport={fitViewport}
          onResetZoom={resetZoom}
          onZoomIn={zoomIn}
          onZoomOut={zoomOut}
          onApplyCommand={services.applyEditorCommand}
          onApplyLiveTransform={services.applyEditorLiveTransform}
          onViewportChange={updateViewport}
          onSelectEntity={selectEntity}
        />
      </div>
    </div>
  );
}
