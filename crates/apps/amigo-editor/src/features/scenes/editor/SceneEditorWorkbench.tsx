import { useCallback, useMemo, useRef, useState } from "react";
import { FileCode2, FileText } from "lucide-react";
import type { EditorFrameDto } from "../../../api/dto";
import type { EditorComponentProps } from "../../../editor-components/componentTypes";
import { DebugSourceLabel } from "../../../debug/debugSource";
import type { WorkspaceRuntimeServices } from "../../../main-window/workspaceRuntimeServices";
import { sceneYamlSource } from "../../files/yamlSourceRefs";
import { buildSceneEditorModel } from "./sceneEditorModel";
import type {
  SceneEditorMode,
  SceneEditorTool,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import {
  selectSceneCanvasEngine,
  selectSceneCanvasKind,
} from "./canvas/selectSceneCanvasEngine";
import {
  clampZoom,
  fitResolutionToViewport,
} from "./sceneEditorTransforms";
import { SceneEditorToolbar } from "./SceneEditorToolbar";

export function SceneEditorWorkbench({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const selectedScene = services.selectedScene ?? null;
  const selectedEntityId = services.selectedEntity?.id ?? null;
  const [mode, setMode] = useState<SceneEditorMode>("edit");
  const [tool, setTool] = useState<SceneEditorTool>("select");
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
  const effectiveFrame = useMemo(() => {
    if (services.editorFrame) {
      return services.editorFrame;
    }
    if (!selectedScene || !services.preview) {
      return null;
    }
    const imageUrl = services.preview.imageUrl ?? services.preview.frameUrls[0] ?? null;
    if (!imageUrl) {
      return null;
    }
    const previewFrame: EditorFrameDto = {
      sessionId: `preview-${selectedScene.id}`,
      revision: 0,
      transport: "image-url",
      width: services.preview.width,
      height: services.preview.height,
      devicePixelRatio: 1,
      imageUrl,
      renderTimeMs: null,
      encodedBytes: null,
    };
    return previewFrame;
  }, [services.editorFrame, services.preview, selectedScene]);
  const Canvas = canvasEngine.render;

  const changeMode = useCallback((nextMode: SceneEditorMode) => {
    setMode(nextMode);
    void services.setEditorMode?.(nextMode);
  }, [services.setEditorMode]);

  const changeTool = useCallback((nextTool: SceneEditorTool) => {
    setTool(nextTool);
    void services.setEditorTool?.(nextTool);
  }, [services.setEditorTool]);

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

  function setZoom(zoom: number) {
    setViewport((current) => ({
      ...current,
      zoom: clampZoom(zoom),
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

  return (
    <div className="scene-editor-workbench">
      <DebugSourceLabel source="src/features/scenes/editor/SceneEditorWorkbench.tsx" />
      <header className="scene-editor-header">
        <div className="scene-editor-title">
          <strong>{selectedScene.label}</strong>
          <small>{selectedScene.documentPath}</small>
        </div>
        <div className="scene-editor-header-actions">
          <SceneEditorToolbar
            engineKind={canvasEngine.kind}
            engineLabel={canvasEngine.label}
            editorModeSession={services.editorModeSession}
            onDiscard={services.discardEditorModeSessionChanges}
            onRedo={services.redoEditorModeTransaction}
            onSave={services.saveEditorModeSession}
            onUndo={services.undoEditorModeTransaction}
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
        <Canvas
          scene={selectedScene}
          canvasKind={canvasEngine.kind}
          frame={effectiveFrame}
          editorModeSession={services.editorModeSession}
          mode={mode}
          model={model}
          snapshot={services.editorSnapshot}
          previewSync={services.editorPreviewSync}
          selectedEntityId={selectedEntityId}
          tool={tool}
          viewport={viewport}
          onModeChange={changeMode}
          onToolChange={changeTool}
          onFitViewport={fitViewport}
          onResetZoom={resetZoom}
          onZoomChange={setZoom}
          onZoomIn={zoomIn}
          onZoomOut={zoomOut}
          onPointerEvent={services.sendEditorPointerEvent}
          onViewportResize={services.resizeEditorModeViewport}
          onViewportChange={updateViewport}
        />
      </div>
    </div>
  );
}
