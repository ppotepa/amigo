import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import { useEditorStore } from "../app/editorStore";
import { emitWorkspaceAttachRequested } from "../app/windowBus";
import { defaultWorkspaceState } from "../app/store/editorState";
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
import { openDetachedWorkspaceWindow } from "../api/windowApi";
import type {
  EditorCommandDto,
  EditorFrameResultDto,
  EditorModeSessionDto,
  EditorProjectFileDto,
  EditorSceneSummaryDto,
} from "../api/dto";
import { DebugSourceProvider, useDebugSourceToggle } from "../debug/debugSource";
import { createComponentInstance, singletonComponentInstanceId } from "../editor-components/componentInstances";
import { editorComponentById } from "../editor-components/componentRegistry";
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
import type { WorkspaceProjectItemOpenResult } from "./workspaceRuntimeServices";
import { fileDiagnosticsFor, findProjectFile, flattenProjectFiles, normalizePath } from "../features/files/fileTreeSelectors";
import type { YamlSourceRef } from "../features/files/yamlSourceRefs";
import { findYamlSourceFile } from "../features/files/yamlSourceRefs";
import { sceneScriptFile } from "../features/scenes/sceneContextModel";
import {
  idleSceneEditorPreviewSync,
} from "../features/scenes/editor/sceneEditorPreviewSync";
import {
  activateEditorTarget as dispatchEditorTargetActivation,
  type EditorTargetIntent,
  type EditorTargetRef,
  type ResolvedEditorTarget,
} from "../editor-targets";
import { componentTabs } from "./workspaceTabs";
import type { OpenWorkspaceEditorRequest } from "./workspaceOpenTypes";
import { componentOpenRequestForProjectFile } from "./workspaceOpenRouting";
import { resolveFileWorkspaceDescriptor } from "../features/files/fileWorkspaceRules";
import { WORKSPACE_DOCK_PROFILES, workspaceDockProfileForComponent } from "./workspaceDockProfiles";
import {
  activeFileFromWorkspaceTab,
  centerComponentInstancesFromTabs,
  openedFilePathsFromTabs,
} from "./workspaceTabAdapters";
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

export type DetachedWorkspaceSurfaceRequest = {
  componentId: string;
  context?: Record<string, string>;
  filePath?: string;
  resourceUri?: string;
  titleOverride?: string;
};

function orderDockInstancesByProfile(
  instances: EditorComponentInstance[],
  profileComponentIds: string[],
): EditorComponentInstance[] {
  if (profileComponentIds.length === 0) {
    return instances;
  }

  const profileRank = new Map(profileComponentIds.map((componentId, index) => [componentId, index]));
  return [...instances].sort((left, right) => {
    const leftRank = profileRank.get(left.componentId) ?? Number.MAX_SAFE_INTEGER;
    const rightRank = profileRank.get(right.componentId) ?? Number.MAX_SAFE_INTEGER;
    return leftRank - rightRank;
  });
}

function activeDockInstanceForProfile(
  instances: EditorComponentInstance[],
  activeInstanceId: string,
  profileComponentIds: string[],
): EditorComponentInstance {
  const activeInstance = instances.find((instance) => instance.instanceId === activeInstanceId) ?? null;
  if (!activeInstance) {
    return preferredDockInstance(instances, profileComponentIds);
  }

  if (profileComponentIds.length === 0 || profileComponentIds.includes(activeInstance.componentId)) {
    return activeInstance;
  }

  return preferredDockInstance(instances, profileComponentIds);
}

function preferredDockInstance(
  instances: EditorComponentInstance[],
  profileComponentIds: string[],
): EditorComponentInstance {
  return (
    instances.find((instance) => profileComponentIds.includes(instance.componentId)) ??
    instances[0]
  );
}

function preferredDockInstanceId(
  instances: EditorComponentInstance[],
  profileComponentIds: string[],
): string | null {
  return preferredDockInstance(instances, profileComponentIds)?.instanceId ?? null;
}

export function MainEditorWindow({
  detachedSurface,
  workspaceId = "main",
}: {
  detachedSurface?: DetachedWorkspaceSurfaceRequest | null;
  workspaceId?: string;
}) {
  const {
    state,
    closeCenterComponentTab,
    closeWorkspaceTab,
    createExpectedFolder,
    focusComponent,
    loadEditorModeSceneHierarchy,
    loadSceneHierarchy,
    markWorkspaceTabDetached,
    openCenterComponentTab,
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
    setWorkspaceDockProfile,
    setWorkspaceDockLayout,
    selectWorkspaceTab,
    setFileDirty,
    setPreviewPlaying,
    validateSelectedMod,
  } = useEditorStore();
  const { activeThemeId } = useThemeService();
  const [componentMenuOpen, setComponentMenuOpen] = useState(false);
  const workspaceState = state.workspaces[workspaceId] ?? state.workspaces.main ?? defaultWorkspaceState({ workspaceId, selection: state.selection });
  const {
    bottomInstanceId,
    dockSizes,
    leftInstanceId,
    resetDockSize,
    resetLayout,
    resizeDock,
    rightBottomInstanceId,
    rightTopInstanceId,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightBottomInstanceId,
    setRightTopInstanceId,
  } = useWorkspaceLayout(workspaceId, workspaceState.dockLayout);
  const [eventFilter, setEventFilter] = useState<string>("all");
  const [eventSessionFilter, setEventSessionFilter] = useState<string>("all");
  const [eventSourceFilter, setEventSourceFilter] = useState<string>("all");
  const [eventSearch, setEventSearch] = useState("");
  const previewSyncRevisionRef = useRef(0);
  const [editorPreviewSync, setEditorPreviewSync] = useState(idleSceneEditorPreviewSync());
  const [currentEditorTarget, setCurrentEditorTarget] = useState<ResolvedEditorTarget | null>(null);
  const { showDebugSources: showComponentSources, setShowDebugSources } = useDebugSourceToggle();

  const details = state.modDetails;
  const session = state.activeSession;
  const workspaceScopedState = useMemo(
    () => ({ ...state, selection: workspaceState.selection }),
    [state, workspaceState.selection],
  );
  const selectedSceneValue = selectSelectedScene(workspaceScopedState);
  const projectTree = details ? state.projectTrees[details.id] : undefined;
  const projectStructureTree = details ? state.projectStructureTrees[details.id] : undefined;
  const projectTreeTask = details ? state.tasks[`project-tree:${details.id}`] : undefined;
  const selectedFileValue = selectSelectedFile(workspaceScopedState, projectTree);
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
    selectSceneEntity: (entityId) => selectSceneEntity(entityId, workspaceId),
    selectUiNode: (selection) => selectUiNode(selection, workspaceId),
  });
  const hierarchy = selectSelectedHierarchy(details, selectedSceneValue, state.sceneHierarchies);
  const hierarchyTask = details && selectedSceneValue ? state.tasks[`scene-hierarchy:${details.id}:${selectedSceneValue.id}`] : undefined;
  const selectedEntityValue = selectSelectedEntity(workspaceScopedState, hierarchy);
  const selectedUiNodeValue = selectSelectedUiNode(workspaceScopedState, hierarchy);
  const selectedUiNodeObjectValue = selectSelectedUiNodeObject(workspaceScopedState, editorSnapshot);
  const selectedAssetValue = selectSelectedAsset(workspaceScopedState, projectTree);
  const resolvedSelection = selectResolvedSelection(workspaceScopedState, projectTree, selectedFileContent ?? null);
  const componentContext: EditorComponentContext = {
    sessionId: session?.sessionId ?? null,
    modId: details?.id ?? session?.modId ?? null,
    selectedSceneId: selectedSceneValue?.id ?? null,
    selectedEntityId: selectedEntityValue?.id ?? null,
    selectedAssetId: selectedAssetValue?.assetKey ?? null,
    capabilities: details?.capabilities ?? [],
  };
  const workspaceCenterComponentTabs = useMemo(
    () => centerComponentInstancesFromTabs({
      sessionId: session?.sessionId,
      tabs: workspaceState.tabs,
    }),
    [session?.sessionId, workspaceState.tabs],
  );
  const openedFilePaths = useMemo(
    () => openedFilePathsFromTabs(workspaceState.tabs),
    [workspaceState.tabs],
  );

  const leftDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "project.explorer", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "assets.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "ui.document.structure", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "scenes.browser", placement: { kind: "leftDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const rightTopDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "entity.inspector", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: SCENE_CONTEXT_COMPONENT_ID, placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
      createComponentInstance({ componentId: "entity.properties", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
    ],
    [session?.sessionId],
  );
  const rightBottomDockInstances = useMemo(
    () => [
      createComponentInstance({ componentId: "target.context", placement: { kind: "rightDock" }, sessionId: session?.sessionId }),
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
    if (workspaceId !== "main") {
      await closeCurrentWindow();
      return;
    }

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

  const handleAttachWorkspace = useCallback(async () => {
    if (workspaceId === "main") {
      return;
    }

    const tabId = workspaceState.activeTabId;
    await emitWorkspaceAttachRequested({
      sourceWorkspaceId: workspaceId,
      targetWorkspaceId: "main",
      tabId,
    }, session?.sessionId ?? null);
    await closeCurrentWindow();
  }, [session?.sessionId, workspaceId, workspaceState.activeTabId]);

  const handleSelectProjectFile = useCallback((file: EditorProjectFileDto) => {
    const matchingScene = details?.scenes.find((scene) => {
      const normalizedDocument = normalizePath(scene.documentPath);
      const normalizedScript = normalizePath(scene.scriptPath);
      return normalizedDocument.endsWith(file.relativePath) || normalizedScript.endsWith(file.relativePath);
    });
    if (matchingScene) {
      void selectScene(matchingScene, workspaceId);
    }
    selectProjectFile(file, workspaceId);
    selectWorkspaceTab(`file:${file.relativePath}`, workspaceId);
  }, [details?.scenes, selectProjectFile, selectScene, selectWorkspaceTab, workspaceId]);

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
    selectWorkspaceTab(SCENE_PREVIEW_TAB_ID, workspaceId);
    setRightTopInstanceId(SCENE_CONTEXT_INSTANCE_ID);
    focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
    await selectScene(scene, workspaceId);
    recordEvent({
      type: "SceneContextActivated",
      sceneId: scene.id,
      sceneLabel: scene.label,
    });
  };

  const fileDiagnostics = selectedFileValue ? fileDiagnosticsFor(selectedFileValue, selectedFileContent) : [];
  const allProblems = [...problems, ...fileDiagnostics];
  const activeFileTabPath = workspaceState.activeTabId.startsWith("file:")
    ? workspaceState.activeTabId.slice("file:".length)
    : null;
  const activeFile = activeFileFromWorkspaceTab({
    activeTabId: workspaceState.activeTabId,
    projectTree,
    selectedFile: selectedFileValue,
  });
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
    closeCenterComponent,
    openCenterComponent,
  } = useCenterComponentTabs({
    activeWorkspaceTabId: workspaceState.activeTabId,
    centerComponentTabs: workspaceCenterComponentTabs,
    closeCenterComponentTab: (instanceId) => closeCenterComponentTab(instanceId, workspaceId),
    detailsId: details?.id ?? null,
    focusComponent,
    openCenterComponentTab: (instance) => openCenterComponentTab(instance, workspaceId),
    openComponent,
    scenePreviewComponentId: SCENE_PREVIEW_COMPONENT_ID,
    scenePreviewInstanceId: SCENE_PREVIEW_INSTANCE_ID,
    scenePreviewTabId: SCENE_PREVIEW_TAB_ID,
    selectWorkspaceTab: (tabId) => selectWorkspaceTab(tabId, workspaceId),
    sessionId: session?.sessionId ?? null,
  });
  const activeWorkspaceSurfaceComponentId = workspaceState.activeTabId === SCENE_PREVIEW_TAB_ID
    ? SCENE_PREVIEW_COMPONENT_ID
    : workspaceState.activeTabId.startsWith("file:")
      ? activeFileDescriptor?.componentId ?? null
      : activeCenterComponent?.componentId ?? null;
  const activeWorkspaceDockProfile = workspaceDockProfileForComponent(
    activeWorkspaceSurfaceComponentId ? editorComponentById(activeWorkspaceSurfaceComponentId) : null,
  );
  const workspaceDockProfile = WORKSPACE_DOCK_PROFILES[workspaceState.dockProfileId] ?? activeWorkspaceDockProfile;
  useEffect(() => {
    if (activeWorkspaceDockProfile.id !== workspaceState.dockProfileId) {
      setWorkspaceDockProfile(workspaceId, activeWorkspaceDockProfile.id);
    }
  }, [activeWorkspaceDockProfile.id, setWorkspaceDockProfile, workspaceId, workspaceState.dockProfileId]);
  useEffect(() => {
    const nextDockLayout = {
      leftDock: {
        ...workspaceState.dockLayout.leftDock,
        activeTabId: leftInstanceId,
        size: dockSizes.leftWidth,
      },
      rightTopDock: {
        ...workspaceState.dockLayout.rightTopDock,
        activeTabId: rightTopInstanceId,
        size: dockSizes.rightWidth,
      },
      rightBottomDock: {
        ...workspaceState.dockLayout.rightBottomDock,
        activeTabId: rightBottomInstanceId,
        size: dockSizes.rightBottomHeight,
      },
      bottomDock: {
        ...workspaceState.dockLayout.bottomDock,
        activeTabId: bottomInstanceId,
        size: dockSizes.bottomHeight,
      },
    };

    if (JSON.stringify(nextDockLayout) !== JSON.stringify(workspaceState.dockLayout)) {
      setWorkspaceDockLayout(workspaceId, nextDockLayout);
    }
  }, [
    bottomInstanceId,
    dockSizes.bottomHeight,
    dockSizes.leftWidth,
    dockSizes.rightBottomHeight,
    dockSizes.rightWidth,
    leftInstanceId,
    rightBottomInstanceId,
    rightTopInstanceId,
    setWorkspaceDockLayout,
    workspaceId,
    workspaceState.dockLayout,
  ]);
  const profiledLeftDockInstances = useMemo(
    () => orderDockInstancesByProfile(leftDockInstances, workspaceDockProfile.left),
    [leftDockInstances, workspaceDockProfile.left],
  );
  const profiledRightTopDockInstances = useMemo(
    () => orderDockInstancesByProfile(rightTopDockInstances, workspaceDockProfile.rightTop),
    [rightTopDockInstances, workspaceDockProfile.rightTop],
  );
  const profiledRightBottomDockInstances = useMemo(
    () => orderDockInstancesByProfile(rightBottomDockInstances, workspaceDockProfile.rightBottom),
    [rightBottomDockInstances, workspaceDockProfile.rightBottom],
  );
  const profiledBottomDockInstances = useMemo(
    () => orderDockInstancesByProfile(bottomDockInstances, workspaceDockProfile.bottom),
    [bottomDockInstances, workspaceDockProfile.bottom],
  );
  const activeLeftInstance = activeDockInstanceForProfile(profiledLeftDockInstances, leftInstanceId, workspaceDockProfile.left);
  const activeRightTopInstance = activeDockInstanceForProfile(profiledRightTopDockInstances, rightTopInstanceId, workspaceDockProfile.rightTop);
  const activeRightBottomInstance = activeDockInstanceForProfile(profiledRightBottomDockInstances, rightBottomInstanceId, workspaceDockProfile.rightBottom);
  const activeBottomInstance = activeDockInstanceForProfile(profiledBottomDockInstances, bottomInstanceId, workspaceDockProfile.bottom);
  useEffect(() => {
    const nextLeftInstanceId = preferredDockInstanceId(profiledLeftDockInstances, workspaceDockProfile.left);
    const nextRightTopInstanceId = preferredDockInstanceId(profiledRightTopDockInstances, workspaceDockProfile.rightTop);
    const nextRightBottomInstanceId = preferredDockInstanceId(profiledRightBottomDockInstances, workspaceDockProfile.rightBottom);
    const nextBottomInstanceId = preferredDockInstanceId(profiledBottomDockInstances, workspaceDockProfile.bottom);

    if (nextLeftInstanceId) setLeftInstanceId(nextLeftInstanceId);
    if (nextRightTopInstanceId) setRightTopInstanceId(nextRightTopInstanceId);
    if (nextRightBottomInstanceId) setRightBottomInstanceId(nextRightBottomInstanceId);
    if (nextBottomInstanceId) setBottomInstanceId(nextBottomInstanceId);
  }, [
    profiledBottomDockInstances,
    profiledLeftDockInstances,
    profiledRightBottomDockInstances,
    profiledRightTopDockInstances,
    setBottomInstanceId,
    setLeftInstanceId,
    setRightBottomInstanceId,
    setRightTopInstanceId,
    workspaceDockProfile,
  ]);
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
      selectWorkspaceTab(SCENE_PREVIEW_TAB_ID, workspaceId);
      focusComponent(SCENE_PREVIEW_INSTANCE_ID, SCENE_PREVIEW_COMPONENT_ID);
      await selectScene(scene, workspaceId);
    },
    [focusComponent, selectScene, selectWorkspaceTab, workspaceId],
  );

  const openUiDocumentEditor = useCallback(
    (target?: {
      sceneId?: string;
      entityId?: string;
      componentIndex?: number;
      focusPath?: string;
      preferredEntityId?: string;
      initialTemplate?: string;
      titleOverride?: string;
    }) => {
      const sceneId = target?.sceneId ?? selectedSceneValue?.id ?? "";
      const context: Record<string, string> = {
        sceneId,
        initialTemplate: target?.initialTemplate ?? "empty-document",
      };
      if (target?.entityId) {
        context.entityId = target.entityId;
      }
      if (target?.componentIndex != null) {
        context.componentIndex = String(target.componentIndex);
      }
      if (target?.focusPath) {
        context.focusPath = target.focusPath;
      }
      if (target?.preferredEntityId) {
        context.preferredEntityId = target.preferredEntityId;
      }

      openCenterComponent("ui.document.editor", {
        context,
        titleOverride: target?.titleOverride ?? "UI Document",
      });
      if (target?.entityId && target.componentIndex != null) {
        recordEvent({
          type: "UiDocumentEditorOpened",
          sceneId,
          entityId: target.entityId,
          componentIndex: target.componentIndex,
        });
      }
    },
    [openCenterComponent, recordEvent, selectedSceneValue?.id],
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
            focusPath: request.focusPath,
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

  const consumedDetachedSurfaceRef = useRef(false);
  useEffect(() => {
    if (!detachedSurface || consumedDetachedSurfaceRef.current || !session?.sessionId) {
      return;
    }

    if (detachedSurface.filePath) {
      const file = projectTree ? findProjectFile(projectTree.root, detachedSurface.filePath) : null;
      if (file) {
        consumedDetachedSurfaceRef.current = true;
        openProjectFileEditor(file);
      }
      return;
    }

    if (detachedSurface.componentId === SCENE_PREVIEW_COMPONENT_ID && detachedSurface.context?.sceneId && details) {
      const scene = details.scenes.find((candidate) => candidate.id === detachedSurface.context?.sceneId);
      if (scene) {
        consumedDetachedSurfaceRef.current = true;
        void openSceneEditor(scene);
      }
      return;
    }

    consumedDetachedSurfaceRef.current = true;
    openCenterComponent(detachedSurface.componentId, {
      context: detachedSurface.context,
      resourceUri: detachedSurface.resourceUri,
      titleOverride: detachedSurface.titleOverride,
    });
  }, [
    detachedSurface,
    details,
    openCenterComponent,
    openProjectFileEditor,
    openSceneEditor,
    projectTree,
    session?.sessionId,
  ]);

  const canDetachWorkspaceTab = useCallback(
    (tabId: string) => {
      if (!session?.sessionId || workspaceId !== "main") {
        return false;
      }

      if (tabId === SCENE_PREVIEW_TAB_ID) {
        return Boolean(editorComponentById(SCENE_PREVIEW_COMPONENT_ID)?.surface?.detachedMode);
      }

      if (tabId.startsWith("file:")) {
        const relativePath = tabId.slice("file:".length);
        const file =
          (projectTree ? findProjectFile(projectTree.root, relativePath) : null) ??
          (selectedFileValue?.relativePath === relativePath ? selectedFileValue : null);
        if (!file) {
          return false;
        }

        const descriptor = resolveFileWorkspaceDescriptor(file);
        return Boolean(editorComponentById(descriptor.componentId)?.surface?.detachedMode);
      }

      const component = workspaceCenterComponentTabs.find((instance) => instance.instanceId === tabId);
      return Boolean(component && editorComponentById(component.componentId)?.surface?.detachedMode);
    },
    [projectTree, selectedFileValue, session?.sessionId, workspaceCenterComponentTabs, workspaceId],
  );

  const detachWorkspaceTab = useCallback(
    (tabId: string) => {
      if (!session?.sessionId || !canDetachWorkspaceTab(tabId)) {
        return;
      }

      if (tabId.startsWith("file:")) {
        const relativePath = tabId.slice("file:".length);
        const file =
          (projectTree ? findProjectFile(projectTree.root, relativePath) : null) ??
          (selectedFileValue?.relativePath === relativePath ? selectedFileValue : null);
        if (!file) {
          return;
        }

        const request = componentOpenRequestForProjectFile(file);
        if (request.kind !== "component") {
          return;
        }

        const title = request.titleOverride ?? file.name;
        const detachedWorkspaceId = `detached-file-${normalizePath(file.relativePath).replace(/[^a-z0-9_-]+/gi, "-")}`;
        void openDetachedWorkspaceWindow({
          sessionId: session.sessionId,
          workspaceId: detachedWorkspaceId,
          title,
          componentId: request.componentId,
          context: request.context,
          filePath: file.relativePath,
          resourceUri: request.resourceUri,
          titleOverride: title,
        })
          .then(() => {
            markWorkspaceTabDetached(workspaceId, tabId, detachedWorkspaceId);
          })
          .catch(reportWindowOpenError);
        return;
      }

      const component =
        tabId === SCENE_PREVIEW_TAB_ID
          ? scenePreviewComponent
          : workspaceCenterComponentTabs.find((instance) => instance.instanceId === tabId);
      if (!component) {
        return;
      }

      const definition = editorComponentById(component.componentId);
      const title = component.titleOverride ?? definition?.title ?? "Workspace";
      const detachedWorkspaceId = `detached-${component.instanceId.replace(/[^a-z0-9_-]+/gi, "-")}`;

      void openDetachedWorkspaceWindow({
        sessionId: session.sessionId,
        workspaceId: detachedWorkspaceId,
        title,
        componentId: component.componentId,
        context: component.context,
        resourceUri: component.resourceUri,
        titleOverride: component.titleOverride ?? title,
      })
        .then(() => {
          markWorkspaceTabDetached(workspaceId, tabId, detachedWorkspaceId);
        })
        .catch(reportWindowOpenError);
    },
    [
      canDetachWorkspaceTab,
      markWorkspaceTabDetached,
      projectTree,
      scenePreviewComponent,
      selectedFileValue,
      session?.sessionId,
      workspaceCenterComponentTabs,
      workspaceId,
    ],
  );

  const workspaceTabs = useWorkspaceTabs({
    centerComponentTabs: workspaceCenterComponentTabs,
    dirtyFiles: state.dirtyFiles,
    editorModeDirty: Boolean(editorModeSession?.dirty),
    openedFilePaths,
    projectTree,
    scenePreviewTabId: SCENE_PREVIEW_TAB_ID,
    selectedScene: selectedSceneValue ?? null,
    workspaceTabs: workspaceState.tabs,
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

  // @codemap anchor:workspace-current-editor-target domain:workspace role:dispatcher priority:P1 layer:app tags:editor-target,selection,right-dock
  function handleActivateEditorTarget(target: EditorTargetRef, intent: EditorTargetIntent) {
    const result = dispatchEditorTargetActivation(target, intent, workspaceRuntimeServices);
    setCurrentEditorTarget(result.resolved);
setRightTopInstanceId("entity.properties:singleton");
    recordEvent({
      type: "EditorTargetActivated",
      targetKind: target.kind,
      targetLabel: result.resolved.descriptor.label,
      intent,
      status: result.resolved.status,
    });
  }
  const workspaceRuntimeServices = useWorkspaceRuntimeServices({
    allProblems,
    details,
    currentEditorTarget,
    activateEditorTarget: handleActivateEditorTarget,
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
    targetBridge: {
      handleSelectProjectFile: openProjectFileEditor,
      openWorkspaceEditor,
      openProjectItemResult,
      openProjectFileEditor,
      openSceneEditor,
      openUiDocumentEditor,
      showYamlView,
      openSceneScript,
      handleSelectAsset: (asset) => selectAsset(asset, workspaceId),
      activateSceneContext,
      selectScene,
      selectSceneEntity,
      selectUiNode,
      openComponent: openWorkspaceComponent,
      showBottomPanel: setBottomInstanceId,
    },
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
        onAttachWorkspace={workspaceId !== "main" ? handleAttachWorkspace : undefined}
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
          "--right-bottom-height": `${dockSizes.rightBottomHeight}px`,
          "--bottom-dock-height": `${dockSizes.bottomHeight}px`,
        } as React.CSSProperties}
      >
        <MainWorkspaceDockGrid
          activeBottomInstance={activeBottomInstance}
          activeLeftInstance={activeLeftInstance}
          activeRightBottomInstance={activeRightBottomInstance}
          activeRightTopInstance={activeRightTopInstance}
          bottomDockInstances={profiledBottomDockInstances}
          bottomTabs={componentTabs(profiledBottomDockInstances)}
          componentContext={componentContext}
          leftDockInstances={profiledLeftDockInstances}
          leftTabs={componentTabs(profiledLeftDockInstances)}
          onFocusComponent={focusComponent}
          onRecordDockTabSelected={(dock, tabId) => recordEvent({ type: "DockTabSelected", dock, tabId })}
          onResizeDock={resizeDock}
          onResetDockSize={resetDockSize}
          onSelectBottomInstance={setBottomInstanceId}
          onSelectLeftInstance={setLeftInstanceId}
          onSelectRightBottomInstance={setRightBottomInstanceId}
          onSelectRightTopInstance={setRightTopInstanceId}
          renderComponentToolbar={renderComponentToolbar}
          rightBottomDockInstances={profiledRightBottomDockInstances}
          rightBottomTabs={componentTabs(profiledRightBottomDockInstances)}
          rightTopDockInstances={profiledRightTopDockInstances}
          rightTopTabs={componentTabs(profiledRightTopDockInstances)}
          showComponentSources={showComponentSources}
          toolbarStateFor={toolbarStateFor}
          workspaceRuntimeServices={{
            ...workspaceRuntimeServices,
            onCreateExpectedFolder: createExpectedFolder,
          }}
        />

        <MainWorkspaceCenter
          activeCenterComponent={activeCenterComponent ?? null}
          activeFile={activeFile}
          activeFileComponent={activeFileComponent}
          activeFileContent={activeFileContent ?? null}
          activeTabId={workspaceState.activeTabId}
          canDetachTab={canDetachWorkspaceTab}
          centerComponentTabs={workspaceCenterComponentTabs}
          closeCenterComponent={closeCenterComponent}
          closeWorkspaceTab={(tabId) => closeWorkspaceTab(tabId, workspaceId)}
          componentContext={componentContext}
          detachWorkspaceTab={detachWorkspaceTab}
          scenePreviewComponent={scenePreviewComponent}
          selectWorkspaceTab={(tabId) => selectWorkspaceTab(tabId, workspaceId)}
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
