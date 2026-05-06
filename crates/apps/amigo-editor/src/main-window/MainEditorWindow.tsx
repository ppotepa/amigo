import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
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
import { getModDetails, getProjectTree, openModSettingsWindow, openSettingsWindow, openThemeWindow } from "../api/editorApi";
import type {
  EditorCommandDto,
  EditorFrameResultDto,
  EditorModeSessionDto,
  EditorProjectFileDto,
  EditorSceneSummaryDto,
} from "../api/dto";
import { DebugSourceProvider, useDebugSourceToggle } from "../debug/debugSource";
import { createComponentInstance, singletonComponentInstanceId } from "../editor-components/componentInstances";
import type {
  EditorComponentContext,
  EditorComponentInstance,
} from "../editor-components/componentTypes";
import { toneForActionId } from "../theme/semanticColorRegistry";
import { themeNameForId } from "../theme/themeRegistry";
import { useThemeService } from "../theme/themeService";
import { closeCurrentWindow, toggleFullscreenWindow } from "./windowControls";
import { MainWindowStatusbar } from "./MainWindowStatusbar";
import { MainWindowTitlebar } from "./MainWindowTitlebar";
import { MainWorkspaceCenter } from "./MainWorkspaceCenter";
import { MainWorkspaceDockGrid } from "./MainWorkspaceDockGrid";
import type { WorkspaceProjectItemOpenResult, WorkspaceProjectNodeRef } from "./workspaceRuntimeServices";
import { fileDiagnosticsFor, findProjectFile, flattenProjectFiles, normalizePath } from "../features/files/fileTreeSelectors";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";
import { findYamlSourceFile } from "../features/files/yamlSourceRefs";
import { sceneScriptFile } from "../features/scenes/sceneContextModel";
import {
  idleSceneEditorPreviewSync,
} from "../features/scenes/editor/sceneEditorPreviewSync";
import { PROJECT_NODE_ACTIONS } from "../features/project/projectNodeActions";
import { componentTabs } from "./workspaceTabs";
import type { OpenWorkspaceEditorRequest } from "./workspaceOpenTypes";
import { componentOpenRequestForProjectFile } from "./workspaceOpenRouting";
import { resolveFileWorkspaceDescriptor } from "../features/files/fileWorkspaceRules";
import {
  DEFAULT_WORKSPACE_TOOLBOX_ACTION_IDS,
  WORKSPACE_TOOLBOX_ACTIONS,
  type WorkspaceToolboxAction,
  type WorkspaceToolboxActionId,
} from "./toolboxRegistry";
import { useWorkspaceLayout } from "./useWorkspaceLayout";
import { useApplyEditorCommand } from "./hooks/useApplyEditorCommand";
import { useCenterComponentTabs } from "./hooks/useCenterComponentTabs";
import { useComponentToolbarHost } from "./hooks/useComponentToolbarHost";
import { useEditorModeCommands } from "./hooks/useEditorModeCommands";
import { useEditorModeFrame } from "./hooks/useEditorModeFrame";
import { useEditorModeSessionLifecycle } from "./hooks/useEditorModeSessionLifecycle";
import { useWorkspaceTabs } from "./hooks/useWorkspaceTabs";
import { useWorkspaceRuntimeServices } from "./hooks/useWorkspaceRuntimeServices";
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
    loadEditorModeSceneHierarchy,
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
  const eventRows = state.events.slice(0, 40);
  const windowEventRows = state.windowEvents.slice(0, 12);
  const sceneDiagnostics = selectedSceneValue?.diagnostics ?? [];
  const modDiagnostics = details?.diagnostics ?? [];
  const problems = [...modDiagnostics, ...sceneDiagnostics];
  const {
    applyEditorFrameResult,
    editorFrame,
    editorModeSession,
    editorSnapshot,
    editorSnapshotSceneId,
    setEditorFrame,
    setEditorModeSession,
    setEditorSnapshot,
    setEditorSnapshotSceneId,
  } = useEditorModeFrame({
    selectSceneEntity,
    selectUiNode,
  });
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
      createComponentInstance({ componentId: "document.changes", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
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
  const { renderComponentToolbar, toolbarStateFor } = useComponentToolbarHost({
    modId: details?.id ?? null,
    refreshProjectTree,
  });

  function reportWindowOpenError(error: unknown) {
    window.alert(`Failed to open window: ${error instanceof Error ? error.message : String(error)}`);
  }

  const handleOpenTitlebarComponent = (componentId: string) => {
    openComponent(componentId, {
      modId: details?.id ?? "",
      sceneId: selectedSceneValue?.id ?? "",
      sessionId: session?.sessionId ?? "",
    });
    setComponentMenuOpen(false);
  };

  const handleCloseWorkspace = async () => {
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
  };

  const handleSelectProjectFile = useCallback((file: EditorProjectFileDto) => {
    const matchingScene = details?.scenes.find((scene) => {
      const normalizedDocument = normalizePath(scene.documentPath);
      const normalizedScript = normalizePath(scene.scriptPath);
      return normalizedDocument.endsWith(file.relativePath) || normalizedScript.endsWith(file.relativePath);
    });
    if (matchingScene) {
      void selectScene(matchingScene);
    }
    selectProjectFile(file);
    selectWorkspaceTab(`file:${file.relativePath}`);
  }, [details?.scenes, selectProjectFile, selectScene, selectWorkspaceTab]);

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

  const {
    closeEditorModeSession: closeEditorModeSessionForSelectedScene,
    editorModeError,
    editorModeOpening,
    editorModeSessionRef,
    openEditorModeSession: openEditorModeSessionForSelectedScene,
    refreshEditorSnapshot,
    refreshEditorSnapshotForScene,
  } = useEditorModeSessionLifecycle({
    recordEvent,
    scene: selectedSceneValue ?? null,
    sessionId: session?.sessionId ?? null,
    setEditorFrame,
    setEditorModeSession,
    setEditorSnapshot,
    setEditorSnapshotSceneId,
  });

  const editorModeCommands = useEditorModeCommands({
    applyEditorFrameResult,
    editorModeSessionRef,
    loadEditorModeSceneHierarchy,
    recordEvent,
    sessionId: session?.sessionId ?? null,
  });

  const applyEditorCommand = useApplyEditorCommand({
    applyEditorFrameResult,
    editorModeSessionId: editorModeSession?.editorModeSessionId ?? null,
    loadEditorModeSceneHierarchy,
    loadSceneHierarchy,
    modId: details?.id ?? null,
    previewSyncRevisionRef,
    recordEvent,
    regeneratePreview,
    refreshEditorSnapshotForScene,
    selectedScene: selectedSceneValue ?? null,
    sessionId: session?.sessionId ?? null,
    setEditorPreviewSync,
    setEditorSnapshot,
    setEditorSnapshotSceneId,
  });

  useEffect(() => {
    setEditorPreviewSync(idleSceneEditorPreviewSync(selectedSceneValue?.id ?? null));
  }, [selectedSceneValue?.id]);

  const activeEditorSnapshot = editorSnapshotSceneId === selectedSceneValue?.id ? editorSnapshot : null;

  const refreshSceneHierarchyForSelectedScene = useCallback(async () => {
    const scene = selectedSceneValue;
    if (!scene || !details) return;

    if (session?.sessionId && editorModeSession?.editorModeSessionId) {
      await loadEditorModeSceneHierarchy(session.sessionId, editorModeSession.editorModeSessionId);
      return;
    }

    await loadSceneHierarchy(details.id, scene.id, true);
  }, [
    details,
    editorModeSession?.editorModeSessionId,
    loadEditorModeSceneHierarchy,
    loadSceneHierarchy,
    selectedSceneValue,
    session?.sessionId,
  ]);

  const activateSceneContext = async (scene: EditorSceneSummaryDto) => {
    selectWorkspaceTab(SCENE_PREVIEW_TAB_ID);
    setRightInstanceId(SCENE_CONTEXT_INSTANCE_ID);
    focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
    await selectScene(scene);
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
  const activeFile = activeFileTabPath
    ? (projectTree ? findProjectFile(projectTree.root, activeFileTabPath) : null)
      ?? (selectedFileValue?.relativePath === activeFileTabPath ? selectedFileValue : null)
    : selectedFileValue;
  const activeFileContent = details && activeFile ? state.projectFileContents[`${details.id}:${activeFile.relativePath}`] : undefined;
  const activeFileDescriptor = activeFile ? resolveFileWorkspaceDescriptor(activeFile) : null;
  const activeFileComponent = activeFile && activeFileTabPath ? createComponentInstance({
    componentId: activeFileDescriptor?.componentId ?? "file.binary",
    context: {
      fileKind: activeFileDescriptor?.fileKind ?? activeFile.kind,
      filePath: activeFile.relativePath,
    },
    placement: { kind: "centerTab" },
    resourceUri: activeFile.relativePath,
    sessionId: session?.sessionId,
    titleOverride: activeFile.name,
  }) : null;
  const scenePreviewComponent = createComponentInstance({
    componentId: SCENE_PREVIEW_COMPONENT_ID,
    context: { sceneId: selectedSceneValue?.id ?? "" },
    placement: { kind: "centerTab" },
    sessionId: session?.sessionId,
  });
  const {
    activeCenterComponent,
    centerComponentTabs,
    closeCenterComponent,
    openCenterComponent,
  } = useCenterComponentTabs({
    activeWorkspaceTabId: state.activeWorkspaceTabId,
    detailsId: details?.id ?? null,
    focusComponent,
    openComponent,
    scenePreviewComponentId: SCENE_PREVIEW_COMPONENT_ID,
    scenePreviewInstanceId: SCENE_PREVIEW_INSTANCE_ID,
    scenePreviewTabId: SCENE_PREVIEW_TAB_ID,
    selectWorkspaceTab,
    sessionId: session?.sessionId ?? null,
  });
  const openProjectFileEditor = useCallback(
    (file: EditorProjectFileDto) => {
      const request = componentOpenRequestForProjectFile(file);
      handleSelectProjectFile(file);

      if (request.kind === "component") {
        recordEvent({
          type: "ComponentOpenRequested",
          componentId: request.componentId,
          context: request.context,
        });
      }
    },
    [handleSelectProjectFile, recordEvent],
  );

  const openSceneEditor = useCallback(
    async (scene: EditorSceneSummaryDto) => {
      selectWorkspaceTab(SCENE_PREVIEW_TAB_ID);
      focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
      await selectScene(scene);
    },
    [focusComponent, selectScene, selectWorkspaceTab],
  );

  const openUiDocumentEditor = useCallback(
    (target: {
      sceneId: string;
      entityId: string;
      componentIndex: number;
      titleOverride?: string;
    }) => {
      openCenterComponent("ui.document.editor", {
        context: {
          sceneId: target.sceneId,
          entityId: target.entityId,
          componentIndex: String(target.componentIndex),
        },
        titleOverride: target.titleOverride ?? "UI Document",
      });
      recordEvent({
        type: "UiDocumentEditorOpened",
        sceneId: target.sceneId,
        entityId: target.entityId,
        componentIndex: target.componentIndex,
      });
    },
    [openCenterComponent, recordEvent],
  );

  const openWorkspaceEditor = useCallback(
    (request: OpenWorkspaceEditorRequest) => {
      switch (request.kind) {
        case "component":
          openCenterComponent(request.componentId, {
            context: request.context,
            resourceUri: request.resourceUri,
            titleOverride: request.titleOverride,
          });
          return;
        case "project-file":
          openProjectFileEditor(request.file);
          return;
        case "scene":
          void openSceneEditor(request.scene);
          return;
        case "ui-document":
          openUiDocumentEditor({
            sceneId: request.sceneId,
            entityId: request.entityId,
            componentIndex: request.componentIndex,
            titleOverride: request.titleOverride,
          });
          return;
        case "asset":
          selectAsset(request.asset);
      }
    },
    [
      openCenterComponent,
      openProjectFileEditor,
      openSceneEditor,
      openUiDocumentEditor,
      selectAsset,
    ],
  );

  const openProjectItemResult = useCallback(
    async (result: WorkspaceProjectItemOpenResult) => {
      const modId = details?.id ?? session?.modId ?? null;
      if (!modId) return;

      await validateSelectedMod();
      await refreshProjectTree(modId);

      if (result.selectedSceneId) {
        const freshDetails = await getModDetails(modId);
        const scene = freshDetails.scenes.find((candidate) => candidate.id === result.selectedSceneId);
        if (scene) {
          await openSceneEditor(scene);
          return;
        }
      }

      if (result.selectedFilePath) {
        const freshTree = await getProjectTree(modId);
        const file = flattenProjectFiles(freshTree.root).find(
          (candidate) => candidate.relativePath === result.selectedFilePath,
        );
        if (file) {
          openProjectFileEditor(file);
        }
      }
    },
    [
      details?.id,
      openProjectFileEditor,
      openSceneEditor,
      refreshProjectTree,
      session?.modId,
      validateSelectedMod,
    ],
  );

  const openWorkspaceComponent = (componentId: string, context?: Record<string, string>) => {
    if (
      componentId === "ui.document.editor" &&
      context?.sceneId &&
      context.entityId &&
      context.componentIndex
    ) {
      recordEvent({
        type: "UiDocumentEditorOpened",
        sceneId: context.sceneId,
        entityId: context.entityId,
        componentIndex: Number(context.componentIndex),
      });
    }
    openCenterComponent(componentId, { context });
  };

  const workspaceTabs = useWorkspaceTabs({
    centerComponentTabs,
    dirtyFiles: state.dirtyFiles,
    editorModeDirty: Boolean(editorModeSession?.dirty),
    openedFilePaths: state.openedFilePaths,
    projectTree,
    scenePreviewTabId: SCENE_PREVIEW_TAB_ID,
    selectedScene: selectedSceneValue ?? null,
  });

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
      openCenterComponent: (componentId) => openCenterComponent(componentId),
      showBottomPanel: setBottomInstanceId,
      validateSelectedMod,
    });
  }

  const workspaceRuntimeServices = useWorkspaceRuntimeServices({
    allProblems,
    details,
    editorSnapshot: activeEditorSnapshot,
    editorModeSession,
    editorFrame,
    editorPreviewSync,
    applyEditorCommand,
    recordEvent,
    openEditorModeSession: openEditorModeSessionForSelectedScene,
    closeEditorModeSession: closeEditorModeSessionForSelectedScene,
    resizeEditorModeViewport: editorModeCommands.resizeEditorModeViewport,
    setEditorMode: editorModeCommands.setEditorMode,
    setEditorTool: editorModeCommands.setEditorTool,
    saveEditorModeSession: editorModeCommands.saveEditorModeSession,
    discardEditorModeSessionChanges: editorModeCommands.discardEditorModeSessionChanges,
    undoEditorModeTransaction: editorModeCommands.undoEditorModeTransaction,
    redoEditorModeTransaction: editorModeCommands.redoEditorModeTransaction,
    sendEditorPointerEvent: editorModeCommands.sendEditorPointerEvent,
    refreshEditorSnapshot,
    refreshSceneHierarchy: refreshSceneHierarchyForSelectedScene,
    eventFilter,
    eventRows,
    eventSearch,
    eventSessionFilter,
    eventSourceFilter,
    handleSelectProjectFile: openProjectFileEditor,
    openWorkspaceEditor,
    openProjectItemResult,
    openProjectFileEditor,
    openSceneEditor,
    openUiDocumentEditor,
    showYamlView,
    openSceneScript,
    handleSelectAsset: selectAsset,
    hierarchy,
    hierarchyTask,
    onRevealSelectedFile: () => void revealSelectedProjectFile(),
    onFileDirtyChange: setFileDirty,
    reloadModDetails: async () => {
      await validateSelectedMod();
    },
    onProjectTreeRefresh: async () => {
      if (details) {
        await refreshProjectTree(details.id);
      }
    },
    openComponent: openWorkspaceComponent,
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
  });

  return (
    <DebugSourceProvider value={showComponentSources}>
      <main className="main-window-shell window-shell workspace-window-shell">
      <MainWindowTitlebar
        componentMenuOpen={componentMenuOpen}
        details={details ?? null}
        onCloseWorkspace={handleCloseWorkspace}
        onOpenComponent={handleOpenTitlebarComponent}
        onOpenModSettings={() =>
          void (session ? openModSettingsWindow(session.sessionId) : openSettingsWindow()).catch(reportWindowOpenError)
        }
        onOpenTheme={() => void openThemeWindow().catch(reportWindowOpenError)}
        onToggleComponentMenu={() => setComponentMenuOpen((open) => !open)}
        onToggleDebugSources={() => setShowDebugSources((current) => !current)}
        session={session ? { sessionId: session.sessionId, modId: session.modId, rootPath: session.rootPath } : null}
        showDebugSources={showComponentSources}
      />

      <section
        className="workspace-grid"
        style={{
          "--left-dock-width": `${dockSizes.leftWidth}px`,
          "--right-dock-width": `${dockSizes.rightWidth}px`,
          "--bottom-dock-height": `${dockSizes.bottomHeight}px`,
        } as React.CSSProperties}
      >
        <MainWorkspaceDockGrid
          activeBottomInstance={activeBottomInstance}
          activeLeftInstance={activeLeftInstance}
          activeRightInstance={activeRightInstance}
          bottomDockInstances={bottomDockInstances}
          bottomTabs={componentTabs(bottomDockInstances)}
          componentContext={componentContext}
          leftDockInstances={leftDockInstances}
          leftTabs={componentTabs(leftDockInstances)}
          onFocusComponent={focusComponent}
          onRecordDockTabSelected={(dock, tabId) => recordEvent({ type: "DockTabSelected", dock, tabId })}
          onResizeDock={resizeDock}
          onResetDockSize={resetDockSize}
          onSelectBottomInstance={setBottomInstanceId}
          onSelectLeftInstance={setLeftInstanceId}
          onSelectRightInstance={setRightInstanceId}
          renderComponentToolbar={renderComponentToolbar}
          rightDockInstances={rightDockInstances}
          rightTabs={componentTabs(rightDockInstances)}
          showComponentSources={showComponentSources}
          toolbarStateFor={toolbarStateFor}
          workspaceRuntimeServices={{
            ...workspaceRuntimeServices,
            activateSceneContext,
            onCreateExpectedFolder: createExpectedFolder,
            onProjectNodeActivated: handleProjectNodeActivated,
          }}
        />

        <MainWorkspaceCenter
          activeCenterComponent={activeCenterComponent ?? null}
          activeFile={activeFile}
          activeFileComponent={activeFileComponent}
          activeFileContent={activeFileContent ?? null}
          activeTabId={state.activeWorkspaceTabId}
          centerComponentTabs={centerComponentTabs}
          closeCenterComponent={closeCenterComponent}
          closeWorkspaceTab={closeWorkspaceTab}
          componentContext={componentContext}
          scenePreviewComponent={scenePreviewComponent}
          selectWorkspaceTab={selectWorkspaceTab}
          showComponentSources={showComponentSources}
          tabs={workspaceTabs}
          workspaceRuntimeServices={workspaceRuntimeServices}
        />
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
