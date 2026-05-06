import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import {
  ArrowLeft,
  Box,
  FileCode2,
  Pause,
  Play,
  Settings,
} from "lucide-react";
import { useEditorStore } from "../app/editorStore";
import {
  activePreview as selectActivePreview,
  resolvedSelection as selectResolvedSelection,
  selectedAsset as selectSelectedAsset,
  selectedEntity as selectSelectedEntity,
  selectedFile as selectSelectedFile,
  selectedHierarchy as selectSelectedHierarchy,
  selectedScene as selectSelectedScene,
  selectedUiNode as selectSelectedUiNode,
  selectedUiNodeObject as selectSelectedUiNodeObject,
} from "../app/store/editorSelectors";
import {
  applyEditorCommand as applyEditorCommandApi,
  closeEditorModeSession as closeEditorModeSessionApi,
  discardEditorModeSessionChanges as discardEditorModeSessionChangesApi,
  getEditorSceneSnapshot,
  openEditorModeSession as openEditorModeSessionApi,
  openModSettingsWindow,
  openSettingsWindow,
  resizeEditorModeViewport as resizeEditorModeViewportApi,
  redoEditorModeTransaction as redoEditorModeTransactionApi,
  saveEditorModeSession as saveEditorModeSessionApi,
  sendEditorPointerEvent as sendEditorPointerEventApi,
  setEditorMode as setEditorModeApi,
  setEditorTool as setEditorToolApi,
  undoEditorModeTransaction as undoEditorModeTransactionApi,
  openThemeWindow,
} from "../api/editorApi";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorFrameDto,
  EditorFrameResultDto,
  EditorModeDto,
  EditorModeSessionDto,
  EditorPointerEventDto,
  EditorToolDto,
  EditorViewportDto,
  EditorProjectFileDto,
  EditorSceneSnapshotDto,
  EditorSceneSummaryDto,
} from "../api/dto";
import { DebugSourceProvider, DebugSourceToggleButton, useDebugSourceToggle } from "../debug/debugSource";
import { ComponentToolbar, defaultToolbarState } from "../editor-components/ComponentToolbar";
import { createComponentInstance, singletonComponentInstanceId } from "../editor-components/componentInstances";
import { editorComponentById, iconForEditorComponent } from "../editor-components/componentRegistry";
import type {
  ComponentToolbarState,
  ComponentToolbarValue,
  EditorComponentContext,
  EditorComponentInstance,
} from "../editor-components/componentTypes";
import { ThemeButton } from "../theme/ThemeButton";
import { semanticIconClass, toneForActionId, toneForComponentDomain, toneForFileKind } from "../theme/semanticColorRegistry";
import { themeNameForId } from "../theme/themeRegistry";
import { useThemeService } from "../theme/themeService";
import { closeCurrentWindow, toggleFullscreenWindow } from "./windowControls";
import { ComponentMenu } from "./ComponentMenu";
import { DockAreaHost } from "./DockAreaHost";
import { WorkspaceComponentHost } from "./WorkspaceComponentHost";
import { MainWindowStatusbar } from "./MainWindowStatusbar";
import { WorkspaceResizeHandle } from "./WorkspaceResizeHandle";
import type { WorkspaceRuntimeServices, WorkspaceProjectNodeRef } from "./workspaceRuntimeServices";
import { fileDiagnosticsFor, findProjectFile, normalizePath } from "../features/files/fileTreeSelectors";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";
import { findYamlSourceFile } from "../features/files/yamlSourceRefs";
import { sceneScriptFile } from "../features/scenes/sceneContextModel";
import {
  idleSceneEditorPreviewSync,
  sceneEditorPreviewFailed,
  sceneEditorPreviewReady,
  sceneEditorPreviewRegenerating,
} from "../features/scenes/editor/sceneEditorPreviewSync";
import { PROJECT_NODE_ACTIONS } from "../features/project/projectNodeActions";
import { componentTabs } from "./workspaceTabs";
import { resolveFileWorkspaceDescriptor } from "../features/files/fileWorkspaceRules";
import {
  DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS,
  WORKSPACE_TOOLBOX_ACTIONS,
  type WorkspaceToolboxAction,
  type WorkspaceToolboxActionId,
} from "./toolboxRegistry";
import { useWorkspaceLayout } from "./useWorkspaceLayout";
import "./main-window.css";

function formatTaskTime(value: number): string {
  return new Date(value).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

const SCENE_PREVIEW_TAB_ID = "scene-preview";
const SCENE_PREVIEW_COMPONENT_ID = "scene.preview";
const SCENE_PREVIEW_INSTANCE_ID = singletonComponentInstanceId(SCENE_PREVIEW_COMPONENT_ID);
const SCENE_CONTEXT_COMPONENT_ID = "scene.context";
const SCENE_CONTEXT_INSTANCE_ID = singletonComponentInstanceId(SCENE_CONTEXT_COMPONENT_ID);

export function MainEditorWindow() {
  const {
    state,
    closeWorkspaceTab,
    createExpectedFolder,
    focusComponent,
    loadSceneHierarchy,
    openComponent,
    returnToStartup,
    regeneratePreview,
    recordEvent,
    revealSelectedProjectFile,
    refreshProjectTree,
    selectAsset,
    selectProjectFile,
    selectScene,
    selectSceneEntity,
    selectUiNode,
    selectWorkspaceTab,
    setFileDirty,
    setPreviewPlaying,
    validateSelectedMod,
  } = useEditorStore();
  const { activeThemeId } = useThemeService();
  const [componentMenuOpen, setComponentMenuOpen] = useState(false);
  const {
    bottomInstanceId,
    dockSizes,
    leftInstanceId,
    resetDockSize,
    resetLayout,
    resizeDock,
    rightInstanceId,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightInstanceId,
  } = useWorkspaceLayout();
  const [eventFilter, setEventFilter] = useState<string>("all");
  const [eventSessionFilter, setEventSessionFilter] = useState<string>("all");
  const [eventSourceFilter, setEventSourceFilter] = useState<string>("all");
  const [eventSearch, setEventSearch] = useState("");
  const [centerComponentTabs, setCenterComponentTabs] = useState<EditorComponentInstance[]>([]);
  const [componentToolbarState, setComponentToolbarState] = useState<Record<string, ComponentToolbarState>>({});
  const [editorSnapshot, setEditorSnapshot] = useState<EditorSceneSnapshotDto | null>(null);
  const [editorSnapshotSceneId, setEditorSnapshotSceneId] = useState<string | null>(null);
  const [editorModeSession, setEditorModeSession] = useState<EditorModeSessionDto | null>(null);
  const [editorFrame, setEditorFrame] = useState<EditorFrameDto | null>(null);
  const [editorModeOpening, setEditorModeOpening] = useState(false);
  const [editorModeError, setEditorModeError] = useState<string | null>(null);
  const editorModeSessionRef = useRef<EditorModeSessionDto | null>(null);
  const openingEditorModeSceneRef = useRef<string | null>(null);
  const previewSyncRevisionRef = useRef(0);
  const [editorPreviewSync, setEditorPreviewSync] = useState(idleSceneEditorPreviewSync());
  const { showDebugSources: showComponentSources, setShowDebugSources } = useDebugSourceToggle();

  const details = state.modDetails;
  const session = state.activeSession;
  const selectedSceneValue = selectSelectedScene(state);
  const projectTree = details ? state.projectTrees[details.id] : undefined;
  const projectStructureTree = details ? state.projectStructureTrees[details.id] : undefined;
  const projectTreeTask = details ? state.tasks[`project-tree:${details.id}`] : undefined;
  const selectedFileValue = selectSelectedFile(state, projectTree);
  const selectedFileContent = details && selectedFileValue ? state.projectFileContents[`${details.id}:${selectedFileValue.relativePath}`] : undefined;
  const preview = selectActivePreview(details, selectedSceneValue?.id ?? null, state.previews);
  const previewTask = details && selectedSceneValue ? state.tasks[`preview:${details.id}:${selectedSceneValue.id}`] : undefined;
  const runningTasks = Object.values(state.tasks).filter((task) => task.status === "running");
  const eventRows = state.events.slice(0, 8);
  const windowEventRows = state.windowEvents.slice(0, 12);
  const sceneDiagnostics = selectedSceneValue?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const hierarchy = selectSelectedHierarchy(details, selectedSceneValue, state.sceneHierarchies);
  const hierarchyTask = details && selectedSceneValue ? state.tasks[`scene-hierarchy:${details.id}:${selectedSceneValue.id}`] : undefined;
  const selectedEntityValue = selectSelectedEntity(state, hierarchy);
  const selectedUiNodeValue = selectSelectedUiNode(state, hierarchy);
  const selectedUiNodeObjectValue = selectSelectedUiNodeObject(state, editorSnapshot);
  const selectedAssetValue = selectSelectedAsset(state, projectTree);
  const resolvedSelection = selectResolvedSelection(state, projectTree, selectedFileContent ?? null);
  const componentContext: EditorComponentContext = {
    sessionId: session?.sessionId ?? null,
    modId: details?.id ?? session?.modId ?? null,
    selectedSceneId: selectedSceneValue?.id ?? null,
    selectedEntityId: selectedEntityValue?.id ?? null,
    selectedAssetId: selectedAssetValue?.assetKey ?? null,
    capabilities: details?.capabilities ?? [],
  };
  const leftDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "project.explorer", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "assets.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scenes.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const rightDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "entity.inspector", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: SCENE_CONTEXT_COMPONENT_ID, placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "entity.properties", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "diagnostics.panel", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const bottomDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "files.browser", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scripts.browser", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "diagnostics.problems", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "events.log", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "tasks.monitor", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scripting.console", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "cache.preview", placement: { kind: "bottomDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const activeLeftInstance = leftDockInstances.find((instance) => instance.instanceId === leftInstanceId) ?? leftDockInstances[0];
  const activeRightInstance = rightDockInstances.find((instance) => instance.instanceId === rightInstanceId) ?? rightDockInstances[0];
  const activeBottomInstance = bottomDockInstances.find((instance) => instance.instanceId === bottomInstanceId) ?? bottomDockInstances[0];

  function toolbarStateFor(instance: EditorComponentInstance): ComponentToolbarState {
    const toolbar = editorComponentById(instance.componentId)?.toolbar;
    return {
      ...defaultToolbarState(toolbar),
      ...(componentToolbarState[instance.instanceId] ?? {}),
    };
  }

  function setToolbarValue(instance: EditorComponentInstance, controlId: string, value: ComponentToolbarValue) {
    setComponentToolbarState((current) => ({
      ...current,
      [instance.instanceId]: {
        ...defaultToolbarState(editorComponentById(instance.componentId)?.toolbar),
        ...(current[instance.instanceId] ?? {}),
        [controlId]: value,
      },
    }));
  }

  function runComponentToolbarAction(instance: EditorComponentInstance, controlId: string) {
    if (instance.componentId === "assets.browser" && controlId === "add") {
      setToolbarValue(instance, "addNonce", String(Date.now()));
      return;
    }

    if (instance.componentId === "assets.browser" && controlId === "refresh" && details) {
      setToolbarValue(instance, "refreshNonce", String(Date.now()));
      void refreshProjectTree(details.id);
    }
  }

  function renderComponentToolbar(instance: EditorComponentInstance) {
    const definition = editorComponentById(instance.componentId);
    const toolbar = definition?.toolbar;
    if (!toolbar) return null;
    return (
      <ComponentToolbar
        toolbar={toolbar}
        tone={definition ? toneForComponentDomain(definition.domain) : "neutral"}
        state={toolbarStateFor(instance)}
        onChange={(controlId, value) => setToolbarValue(instance, controlId, value)}
        onAction={(controlId) => runComponentToolbarAction(instance, controlId)}
      />
    );
  }

  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  const handleSelectProjectFile = (file: EditorProjectFileDto) => {
    const matchingScene = details?.scenes.find((scene) => {
      const normalizedDocument = normalizePath(scene.documentPath);
      const normalizedScript = normalizePath(scene.scriptPath);
      return normalizedDocument.endsWith(file.relativePath) || normalizedScript.endsWith(file.relativePath);
    });
    if (matchingScene) {
      void selectScene(matchingScene);
    }
    selectProjectFile(file);
  };

  const showYamlView = (source: YamlSourceRef) => {
    const file = findYamlSourceFile(projectTree?.root, source);

    if (!file) {
      recordEvent({
        type: "YamlSourceMissing",
        path: source.path,
        label: source.label,
      });
      return;
    }

    handleSelectProjectFile(file);
  };

  const openSceneScript = (scene: EditorSceneSummaryDto) => {
    const file = sceneScriptFile(projectTree, scene);
    if (!file) {
      recordEvent({
        type: "SceneScriptMissing",
        sceneId: scene.id,
        scriptPath: scene.scriptPath,
      });
      return;
    }
    handleSelectProjectFile(file);
  };

  async function refreshEditorSnapshotForScene(scene: EditorSceneSummaryDto | null) {
    if (!session?.sessionId || !scene) {
      setEditorSnapshot(null);
      setEditorSnapshotSceneId(null);
      return;
    }

    try {
      const snapshot = await getEditorSceneSnapshot(session.sessionId, scene.id);
      setEditorSnapshot(snapshot);
      setEditorSnapshotSceneId(scene.id);
      recordEvent({
        type: "EditorSnapshotLoaded",
        sceneId: scene.id,
        objects: snapshot.objects.length,
      });
    } catch (reason) {
      setEditorSnapshot(null);
      setEditorSnapshotSceneId(null);
      recordEvent({
        type: "EditorSnapshotUnavailable",
        sceneId: scene.id,
        error: reason instanceof Error ? reason.message : String(reason),
      });
    }
  }

  const refreshEditorSnapshot = async () => {
    await refreshEditorSnapshotForScene(selectedSceneValue ?? null);
  };

  const applyEditorFrameResult = useCallback((result: EditorFrameResultDto | null | undefined) => {
    if (!result) return;
    if (result.session) {
      editorModeSessionRef.current = result.session;
      setEditorModeSession(result.session);
    }
    if (result.snapshot) {
      setEditorSnapshot(result.snapshot);
      setEditorSnapshotSceneId(result.snapshot.sceneId);
      const selectedUiNode = result.snapshot.selection?.selectedUiNode ?? null;
      if (selectedUiNode) {
        selectUiNode({
          entityId: selectedUiNode.entityId,
          componentIndex: selectedUiNode.componentIndex,
          nodePath: selectedUiNode.nodePath,
        });
      } else {
        const selectedEntityId = result.snapshot.selection?.selectedEntityIds[0] ?? null;
        selectSceneEntity(selectedEntityId);
      }
    }
    if (result.frame) {
      setEditorFrame(result.frame);
    }
  }, [selectSceneEntity, selectUiNode]);

  useEffect(() => {
    editorModeSessionRef.current = editorModeSession;
  }, [editorModeSession]);

  const closeEditorModeSessionForSelectedScene = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!session?.sessionId || !currentSession) {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      return;
    }

    try {
      await closeEditorModeSessionApi(session.sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "CloseEditorModeSession",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    } finally {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      setEditorModeOpening(false);
      setEditorModeError(null);
    }
  }, [recordEvent, session?.sessionId]);

  const openEditorModeSessionForSelectedScene = useCallback(async () => {
    const sceneId = selectedSceneValue?.id;
    if (!session?.sessionId || !sceneId) return;
    if (editorModeSessionRef.current?.sceneId === sceneId) return;
    if (openingEditorModeSceneRef.current === sceneId) return;

    openingEditorModeSceneRef.current = sceneId;
    setEditorModeOpening(true);
    setEditorModeError(null);

    try {
      const devicePixelRatio = window.devicePixelRatio || 1;
      const result = await openEditorModeSessionApi(session.sessionId, sceneId, {
        cssWidth: 1280,
        cssHeight: 720,
        renderWidth: Math.round(1280 * devicePixelRatio),
        renderHeight: Math.round(720 * devicePixelRatio),
        devicePixelRatio,
      });
      editorModeSessionRef.current = result.session;
      setEditorModeSession(result.session);
      setEditorFrame(result.frame);
      setEditorSnapshot(result.snapshot);
      setEditorSnapshotSceneId(result.snapshot.sceneId);
    } catch (reason) {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      setEditorModeError(reason instanceof Error ? reason.message : String(reason));
      recordEvent({
        type: "EditorSnapshotUnavailable",
        sceneId,
        error: reason instanceof Error ? reason.message : String(reason),
      });
    } finally {
      if (openingEditorModeSceneRef.current === sceneId) {
        openingEditorModeSceneRef.current = null;
      }
      setEditorModeOpening(false);
    }
  }, [recordEvent, selectedSceneValue?.id, session?.sessionId]);

  const saveEditorModeSessionForSelectedScene = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!session?.sessionId || !currentSession) return;
    try {
      const result = await saveEditorModeSessionApi(session.sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "SaveEditorModeSession",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    }
  }, [applyEditorFrameResult, recordEvent, session?.sessionId]);

  const discardEditorModeSessionChangesForSelectedScene = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!session?.sessionId || !currentSession) return;
    try {
      const result = await discardEditorModeSessionChangesApi(session.sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "DiscardEditorModeSessionChanges",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    }
  }, [applyEditorFrameResult, recordEvent, session?.sessionId]);

  const undoEditorModeTransactionForSelectedScene = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!session?.sessionId || !currentSession) return;
    try {
      const result = await undoEditorModeTransactionApi(
        session.sessionId,
        currentSession.editorModeSessionId,
      );
      applyEditorFrameResult(result);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "UndoEditorModeTransaction",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    }
  }, [applyEditorFrameResult, recordEvent, session?.sessionId]);

  const redoEditorModeTransactionForSelectedScene = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!session?.sessionId || !currentSession) return;
    try {
      const result = await redoEditorModeTransactionApi(
        session.sessionId,
        currentSession.editorModeSessionId,
      );
      applyEditorFrameResult(result);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "RedoEditorModeTransaction",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    }
  }, [applyEditorFrameResult, recordEvent, session?.sessionId]);

  const resizeEditorModeViewportForSelectedScene = useCallback(
    async (viewport: EditorViewportDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!session?.sessionId || !currentSession) return null;
      try {
        const result = await resizeEditorModeViewportApi(
          session.sessionId,
          currentSession.editorModeSessionId,
          viewport,
        );
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordEvent({
          type: "EditorCommandFailed",
          command: "ResizeEditorModeViewport",
          error: reason instanceof Error ? reason.message : String(reason),
        });
        return null;
      }
    },
    [session?.sessionId, applyEditorFrameResult, recordEvent],
  );

  const setEditorModeForSelectedScene = useCallback(
    async (mode: EditorModeDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!session?.sessionId || !currentSession) return null;
      try {
        const result = await setEditorModeApi(
          session.sessionId,
          currentSession.editorModeSessionId,
          mode,
        );
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordEvent({
          type: "EditorCommandFailed",
          command: "SetEditorMode",
          error: reason instanceof Error ? reason.message : String(reason),
        });
        return null;
      }
    },
    [session?.sessionId, applyEditorFrameResult, recordEvent],
  );

  const setEditorToolForSelectedScene = useCallback(
    async (tool: EditorToolDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!session?.sessionId || !currentSession) return null;
      try {
        const result = await setEditorToolApi(
          session.sessionId,
          currentSession.editorModeSessionId,
          tool,
        );
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordEvent({
          type: "EditorCommandFailed",
          command: "SetEditorTool",
          error: reason instanceof Error ? reason.message : String(reason),
        });
        return null;
      }
    },
    [session?.sessionId, applyEditorFrameResult, recordEvent],
  );

  const sendEditorPointerEvent = useCallback(
    async (event: EditorPointerEventDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!session?.sessionId || !currentSession) return null;
      try {
        const result = await sendEditorPointerEventApi(
          session.sessionId,
          currentSession.editorModeSessionId,
          event,
        );
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordEvent({
          type: "EditorCommandFailed",
          command: "SendEditorPointerEvent",
          error: reason instanceof Error ? reason.message : String(reason),
        });
        return null;
      }
    },
    [session?.sessionId, applyEditorFrameResult, recordEvent],
  );

  async function regeneratePreviewForEditorCommand({
    modId,
    revision,
    sceneId,
  }: {
    modId: string;
    revision: number;
    sceneId: string;
  }) {
    try {
      await regeneratePreview(modId, sceneId, true);
      if (previewSyncRevisionRef.current !== revision) return;
      setEditorPreviewSync(sceneEditorPreviewReady({ sceneId, revision }));
      recordEvent({
        type: "EditorPreviewRegenerated",
        sceneId,
        revision,
      });
    } catch (reason) {
      if (previewSyncRevisionRef.current !== revision) return;
      const message = reason instanceof Error ? reason.message : String(reason);
      setEditorPreviewSync(sceneEditorPreviewFailed({ sceneId, revision, message }));
      recordEvent({
        type: "EditorPreviewRegenerationFailed",
        sceneId,
        revision,
        error: message,
      });
    }
  }

  const applyEditorCommand = async (command: EditorCommandDto): Promise<EditorCommandResultDto | null> => {
    if (!session?.sessionId) return null;
    try {
      const result = await applyEditorCommandApi(session.sessionId, command);
      if (result.snapshot) {
        setEditorSnapshot(result.snapshot);
        setEditorSnapshotSceneId(result.snapshot.sceneId);
      }
      if (
        result.ok &&
        details?.id &&
        (command.type === "SetEntityTransform2D"
          || command.type === "SetTileMapMarker2D"
          || command.type === "SetAttachedLocalOffset2D"
          || command.type === "SetUiNodeProperty")
      ) {
        if (command.type === "SetUiNodeProperty") {
          await loadSceneHierarchy(details.id, command.sceneId, true);
        }
        if (!result.snapshot && selectedSceneValue?.id === command.sceneId) {
          await refreshEditorSnapshotForScene(selectedSceneValue);
        }
        const revision = previewSyncRevisionRef.current + 1;
        previewSyncRevisionRef.current = revision;
        setEditorPreviewSync(sceneEditorPreviewRegenerating({
          sceneId: command.sceneId,
          revision,
        }));
        void regeneratePreviewForEditorCommand({
          modId: details.id,
          sceneId: command.sceneId,
          revision,
        });
      }
      return result;
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: command.type,
        error: reason instanceof Error ? reason.message : String(reason),
      });
      return null;
    }
  };

  useEffect(() => {
    setEditorPreviewSync(idleSceneEditorPreviewSync(selectedSceneValue?.id ?? null));
  }, [selectedSceneValue?.id]);

  useEffect(() => {
    void refreshEditorSnapshotForScene(selectedSceneValue ?? null);
  }, [session?.sessionId, selectedSceneValue?.id]);

  useEffect(() => {
    if (!session?.sessionId || !selectedSceneValue?.id) {
      editorModeSessionRef.current = null;
      openingEditorModeSceneRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      setEditorModeOpening(false);
      setEditorModeError(null);
      return;
    }
    void openEditorModeSessionForSelectedScene();
  }, [session?.sessionId, selectedSceneValue?.id]);

  const activeEditorSnapshot = editorSnapshotSceneId === selectedSceneValue?.id ? editorSnapshot : null;

  const activateSceneContext = async (scene: EditorSceneSummaryDto) => {
    await selectScene(scene);
    selectWorkspaceTab(SCENE_PREVIEW_TAB_ID);
    setRightInstanceId(SCENE_CONTEXT_INSTANCE_ID);
    focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
    recordEvent({
      type: "SceneContextActivated",
      sceneId: scene.id,
      sceneLabel: scene.label,
    });
  };

  const fileDiagnostics = selectedFileValue ? fileDiagnosticsFor(selectedFileValue, selectedFileContent) : [];
  const allProblems = [...problems, ...fileDiagnostics];
  const activeFileTabPath = state.activeWorkspaceTabId.startsWith("file:")
    ? state.activeWorkspaceTabId.slice("file:".length)
    : null;
  const activeFile = activeFileTabPath && projectTree ? findProjectFile(projectTree.root, activeFileTabPath) : selectedFileValue;
  const activeFileContent = details && activeFile ? state.projectFileContents[`${details.id}:${activeFile.relativePath}`] : undefined;
  const activeFileDescriptor = activeFile ? resolveFileWorkspaceDescriptor(activeFile) : null;

  const workspaceTabs = useMemo(() => {
    const sceneDirty = Boolean(editorModeSession?.dirty);
    const tabs: Array<{ id: string; title: string; icon: React.ReactNode; dirty: boolean }> = selectedSceneValue ? [
      {
        id: SCENE_PREVIEW_TAB_ID,
        title: `Scene: ${selectedSceneValue.label}`,
        icon: <Play size={13} className="semantic-icon domain-preview" />,
        dirty: sceneDirty,
      },
    ] : [{
      id: SCENE_PREVIEW_TAB_ID,
      title: "Scene Preview",
      icon: <Play size={13} className="semantic-icon domain-preview" />,
      dirty: sceneDirty,
    }];
    centerComponentTabs.forEach((instance) => {
      const definition = editorComponentById(instance.componentId);
      tabs.push({
        id: instance.instanceId,
        title: instance.titleOverride ?? definition?.title ?? instance.componentId,
        icon: definition ? iconForEditorComponent(definition.icon, 13, toneForComponentDomain(definition.domain)) : <Box size={13} />,
        dirty: false,
      });
    });
    state.openedFilePaths.forEach((relativePath) => {
      const file = projectTree ? findProjectFile(projectTree.root, relativePath) : null;
      if (file) {
        tabs.push({
          id: `file:${file.relativePath}`,
          title: file.name,
          icon: <FileCode2 size={13} className={semanticIconClass(toneForFileKind(file.kind || file.relativePath))} />,
          dirty: Boolean(state.dirtyFiles[file.relativePath]),
        });
      }
    });
    return tabs;
  }, [centerComponentTabs, editorModeSession?.dirty, projectTree, selectedSceneValue, state.dirtyFiles, state.openedFilePaths]);

  function openCenterComponent(componentId: string) {
    if (componentId === SCENE_PREVIEW_COMPONENT_ID) {
      selectWorkspaceTab(SCENE_PREVIEW_TAB_ID);
      focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
      return;
    }

    const instance = createComponentInstance({
      componentId,
      placement: { kind: "centerTab" },
      sessionId: session?.sessionId,
    });
    setCenterComponentTabs((current) => current.some((candidate) => candidate.instanceId === instance.instanceId) ? current : [...current, instance]);
    selectWorkspaceTab(instance.instanceId);
    openComponent(componentId, { modId: details?.id ?? "", sessionId: session?.sessionId ?? "" });
  }

  function closeCenterComponent(instanceId: string) {
    setCenterComponentTabs((current) => current.filter((instance) => instance.instanceId !== instanceId));
    if (state.activeWorkspaceTabId === instanceId) {
      selectWorkspaceTab(SCENE_PREVIEW_TAB_ID);
    }
  }

  function runToolboxAction(actionId: WorkspaceToolboxActionId) {
    recordEvent({ type: "WorkspaceToolboxActionTriggered", actionId });

    if (actionId === "preview.toggle") {
      setPreviewPlaying(!state.previewPlaying);
      return;
    }

    if (actionId === "file.reveal") {
      void revealSelectedProjectFile();
      return;
    }

    if (actionId === "layout.reset") {
      resetLayout();
      recordEvent({ type: "LayoutResetRequested" });
      return;
    }

    if (actionId === "panel.problems") {
      setBottomInstanceId("diagnostics.problems:singleton");
      return;
    }

    if (actionId === "panel.events") {
      setBottomInstanceId("events.log:singleton");
      return;
    }

    if (actionId === "window.fullscreen") {
      void toggleFullscreenWindow();
      return;
    }

    if (actionId === "toolbox.configure") {
      recordEvent({ type: "WorkspaceToolboxConfigureRequested" });
      return;
    }

    if (actionId === "preview.regenerate" && details && selectedSceneValue) {
      void regeneratePreview(details.id, selectedSceneValue.id, true);
      return;
    }

    if (actionId === "mod.validate") {
      void validateSelectedMod();
    }
  }

  const toolboxContext = { details, selectedScene: selectedSceneValue, selectedFile: selectedFileValue };
  const pinnedToolboxActions = DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS
    .map((actionId) => WORKSPACE_TOOLBOX_ACTIONS.find((action) => action.id === actionId))
    .filter((action): action is WorkspaceToolboxAction => Boolean(action));

  function handleProjectNodeActivated(node: WorkspaceProjectNodeRef) {
    if (!details) return;
    recordEvent({ type: "ProjectTreeNodeActivated", modId: details.id, nodeId: node.id, kind: node.kind });
    const action = PROJECT_NODE_ACTIONS.find((candidate) => candidate.canRun(node));
    void action?.run(node, {
      openCenterComponent,
      showBottomPanel: setBottomInstanceId,
      validateSelectedMod,
    });
  }

  const workspaceRuntimeServices: WorkspaceRuntimeServices = {
    allProblems,
    details,
    editorSnapshot: activeEditorSnapshot,
    editorModeSession,
    editorFrame,
    editorPreviewSync,
    applyEditorCommand,
    openEditorModeSession: openEditorModeSessionForSelectedScene,
    closeEditorModeSession: closeEditorModeSessionForSelectedScene,
    resizeEditorModeViewport: resizeEditorModeViewportForSelectedScene,
    setEditorMode: setEditorModeForSelectedScene,
    setEditorTool: setEditorToolForSelectedScene,
    saveEditorModeSession: saveEditorModeSessionForSelectedScene,
    discardEditorModeSessionChanges: discardEditorModeSessionChangesForSelectedScene,
    undoEditorModeTransaction: undoEditorModeTransactionForSelectedScene,
    redoEditorModeTransaction: redoEditorModeTransactionForSelectedScene,
    sendEditorPointerEvent,
    refreshEditorSnapshot,
    eventFilter,
    eventRows,
    eventSearch,
    eventSessionFilter,
    eventSourceFilter,
    handleSelectProjectFile,
    showYamlView,
    openSceneScript,
    handleSelectAsset: selectAsset,
    hierarchy,
    hierarchyTask,
    onRevealSelectedFile: () => void revealSelectedProjectFile(),
    preview,
    previewPlaying: state.previewPlaying,
    previewTask,
    projectTree,
    projectStructureTree,
    projectTreeTask,
    selection: resolvedSelection,
    selectedEntity: selectedEntityValue,
    selectedUiNode: selectedUiNodeValue,
    selectedUiNodeObject: selectedUiNodeObjectValue,
    selectedAsset: selectedAssetValue,
    selectedFile: selectedFileValue,
    selectedFileContent,
    selectedScene: selectedSceneValue,
    selectScene,
    selectSceneEntity,
    selectUiNode,
    setEventFilter,
    setEventSearch,
    setEventSessionFilter,
    setEventSourceFilter,
    tasks: Object.values(state.tasks),
    windowEventRows,
  };

  return (
    <DebugSourceProvider value={showComponentSources}>
      <main className="main-window-shell window-shell workspace-window-shell">
      <header className="main-titlebar window-titlebar">
        <div className="main-brand window-brand">
          <div className="brand-mark">A</div>
          <strong>Amigo Editor</strong>
          <span>{session ? `workspace session ${session.sessionId}` : "workspace"}</span>
        </div>
        <nav className="main-menu" aria-label="Application menu">
          <button type="button">File</button>
          <button type="button">Edit</button>
          <button type="button">View</button>
          <span className="main-menu-popover-anchor">
            <button type="button" onClick={() => setComponentMenuOpen((open) => !open)}>Window</button>
            {componentMenuOpen ? (
              <ComponentMenu
                onOpen={(componentId) => {
                  openComponent(componentId, {
                    modId: details?.id ?? "",
                    sceneId: selectedSceneValue?.id ?? "",
                    sessionId: session?.sessionId ?? "",
                  });
                  setComponentMenuOpen(false);
                }}
              />
            ) : null}
          </span>
          <button type="button" onClick={() => setComponentMenuOpen((open) => !open)}>Tools</button>
        </nav>
        <div className="titlebar-project-context">
          <span className="titlebar-project-summary">
            <strong>{details?.name ?? session?.modId ?? "No mod"}</strong>
            <small>{details ? `${details.id} · ${details.version}` : session?.rootPath ?? "No active session"}</small>
            <span className={`titlebar-status-dot status-${details?.status ?? "warning"}`} aria-label={details?.status ?? "session"} />
          </span>
          <span className="titlebar-separator" aria-hidden="true" />
          <ThemeButton onClick={() => void openThemeWindow().catch(reportWindowOpenError)} />
          <DebugSourceToggleButton showDebugSources={showComponentSources} onToggle={() => setShowDebugSources((current) => !current)} />
          <button
            className="button button-ghost"
            type="button"
            onClick={() =>
              void (session ? openModSettingsWindow(session.sessionId) : openSettingsWindow()).catch(reportWindowOpenError)
            }
          >
            <Settings size={15} />
            Settings
          </button>
          <span className="titlebar-separator" aria-hidden="true" />
          <button
            className="titlebar-action-button"
            type="button"
            onClick={async () => {
              if (state.hasDirtyState) {
                recordEvent({ type: "WorkspaceCloseBlocked", dirtyFileCount: Object.keys(state.dirtyFiles).length });
                const shouldClose = window.confirm("This workspace has unsaved changes. Discard changes and close?");
                if (!shouldClose) {
                  return;
                }
                recordEvent({ type: "WorkspaceCloseConfirmed" });
              }
              await returnToStartup();
              await closeCurrentWindow(session?.sessionId);
            }}
          >
            <ArrowLeft size={15} />
            Close Workspace
          </button>
        </div>
      </header>

      <section
        className="workspace-grid"
        style={{
          "--left-dock-width": `${dockSizes.leftWidth}px`,
          "--right-dock-width": `${dockSizes.rightWidth}px`,
          "--bottom-dock-height": `${dockSizes.bottomHeight}px`,
        } as React.CSSProperties}
      >
        <DockAreaHost
          className="dock-left"
          tabs={componentTabs(leftDockInstances)}
          activeTab={activeLeftInstance.instanceId}
          toolbar={renderComponentToolbar(activeLeftInstance)}
          onSelect={(instanceId) => {
            const instance = leftDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setLeftInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "left", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeLeftInstance}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              activateSceneContext,
              onCreateExpectedFolder: createExpectedFolder,
              onProjectNodeActivated: handleProjectNodeActivated,
              toolbarState: toolbarStateFor(activeLeftInstance),
            }}
          />
        </DockAreaHost>

        <WorkspaceResizeHandle
          className="resize-left"
          orientation="vertical"
          title="Resize left dock"
          onDrag={(delta) => resizeDock("leftWidth", delta)}
          onReset={() => resetDockSize("leftWidth")}
        />

        <section className="workspace-center">
          <div className="workspace-tabs">
            {workspaceTabs.map((tab, index) => (
              <button
                key={tab.id}
                type="button"
                className={`workspace-tab ${state.activeWorkspaceTabId === tab.id ? "active" : ""}`}
                onClick={() => selectWorkspaceTab(tab.id)}
              >
                <span
                  className={`workspace-tab-dirty-dot ${tab.dirty ? "dirty" : "clean"}`}
                  aria-label={tab.dirty ? "Unsaved changes" : "Clean"}
                />
                {tab.icon}
                {tab.title}
                {tab.id.startsWith("file:") || centerComponentTabs.some((instance) => instance.instanceId === tab.id) ? (
                  <span
                    className="workspace-tab-close"
                    role="button"
                    tabIndex={0}
                    onClick={(event) => {
                      event.stopPropagation();
                      if (tab.id.startsWith("file:")) {
                        closeWorkspaceTab(tab.id);
                      } else {
                        closeCenterComponent(tab.id);
                      }
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        event.stopPropagation();
                        if (tab.id.startsWith("file:")) {
                          closeWorkspaceTab(tab.id);
                        } else {
                          closeCenterComponent(tab.id);
                        }
                      }
                    }}
                  >
                    ×
                  </span>
                ) : null}
              </button>
            ))}
          </div>

          {centerComponentTabs.some((instance) => instance.instanceId === state.activeWorkspaceTabId) ? (
            <WorkspaceComponentHost
              instance={centerComponentTabs.find((instance) => instance.instanceId === state.activeWorkspaceTabId)!}
              context={componentContext}
              showDebugSource={showComponentSources}
              services={{ details, selectedFile: selectedFileValue, selectedFileContent, selection: resolvedSelection }}
            />
          ) : activeFile && activeFileTabPath ? (
            <WorkspaceComponentHost
              instance={createComponentInstance({
                componentId: activeFileDescriptor?.componentId ?? "file.binary",
                context: {
                  fileKind: activeFileDescriptor?.fileKind ?? activeFile.kind,
                  filePath: activeFile.relativePath,
                },
                placement: { kind: "centerTab" },
                resourceUri: activeFile.relativePath,
                sessionId: session?.sessionId,
                titleOverride: activeFile.name,
              })}
              context={componentContext}
              showDebugSource={showComponentSources}
              services={{
                details,
                onFileDirtyChange: setFileDirty,
                onProjectTreeRefresh: () => {
                  if (details) {
                    void refreshProjectTree(details.id);
                  }
                },
                onRevealSelectedFile: () => void revealSelectedProjectFile(),
                selection: resolvedSelection,
                selectedFile: activeFile,
                selectedFileContent: activeFileContent,
              }}
            />
          ) : (
            <WorkspaceComponentHost
              instance={createComponentInstance({
                componentId: SCENE_PREVIEW_COMPONENT_ID,
                context: { sceneId: selectedSceneValue?.id ?? "" },
                placement: { kind: "centerTab" },
                sessionId: session?.sessionId,
              })}
              context={componentContext}
              showDebugSource={showComponentSources}
              services={workspaceRuntimeServices}
            />
          )}
        </section>

        <WorkspaceResizeHandle
          className="resize-right"
          orientation="vertical"
          title="Resize right dock"
          onDrag={(delta) => resizeDock("rightWidth", -delta)}
          onReset={() => resetDockSize("rightWidth")}
        />

        <DockAreaHost
          className="dock-right"
          tabs={componentTabs(rightDockInstances)}
          activeTab={activeRightInstance.instanceId}
          toolbar={renderComponentToolbar(activeRightInstance)}
          onSelect={(instanceId) => {
            const instance = rightDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setRightInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "right", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeRightInstance}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              toolbarState: toolbarStateFor(activeRightInstance),
            }}
          />
        </DockAreaHost>

        <WorkspaceResizeHandle
          className="resize-bottom"
          orientation="horizontal"
          title="Resize bottom dock"
          onDrag={(delta) => resizeDock("bottomHeight", -delta)}
          onReset={() => resetDockSize("bottomHeight")}
        />

        <DockAreaHost
          className="dock-bottom"
          tabs={componentTabs(bottomDockInstances)}
          activeTab={activeBottomInstance.instanceId}
          toolbar={renderComponentToolbar(activeBottomInstance)}
          onSelect={(instanceId) => {
            const instance = bottomDockInstances.find((candidate) => candidate.instanceId === instanceId);
            if (!instance) return;
            setBottomInstanceId(instanceId);
            focusComponent(instance.instanceId, instance.componentId);
            recordEvent({ type: "DockTabSelected", dock: "bottom", tabId: instanceId });
          }}
        >
          <WorkspaceComponentHost
            instance={activeBottomInstance}
            context={componentContext}
            showDebugSource={showComponentSources}
            services={{
              ...workspaceRuntimeServices,
              toolbarState: toolbarStateFor(activeBottomInstance),
            }}
          />
        </DockAreaHost>
      </section>

      <MainWindowStatusbar
        activeThemeName={themeNameForId(activeThemeId)}
        details={details ?? null}
        editorModeError={editorModeError}
        editorModeOpening={editorModeOpening}
        runningTaskCount={runningTasks.length}
      />
      </main>
    </DebugSourceProvider>
  );
}
