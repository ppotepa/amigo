import React, { createContext, useCallback, useContext, useEffect, useMemo, useReducer } from "react";
import { closeEditorSession, createExpectedProjectFolder, getEditorSession, getModDetails, getProjectStructureTree, getProjectTree, getSceneHierarchy, listKnownMods, openModWorkspace, readProjectFile, requestScenePreview, revealModFolder, revealProjectFile, revealSceneDocument, validateMod } from "../api/editorApi";
import type { EditorModDetailsDto, EditorModSummaryDto, EditorProjectFileContentDto, EditorProjectFileDto, EditorProjectStructureTreeDto, EditorProjectTreeDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, ManagedAssetDto, OpenModResultDto, ScenePreviewDto } from "../api/dto";
import type { EditorEvent } from "./editorEvents";
import type { EditorTask } from "./editorTasks";
import { createTask, failTask, finishTask } from "./editorTasks";
import { listenPreviewProgress } from "./previewProgressBus";
import { runEditorTask } from "./runEditorTask";
import { selectedFilePath, selectedModId, selectedSceneId } from "./selectionSelectors";
import type { Action } from "./store/editorActions";
import { reducer } from "./store/editorReducer";
import type { EditorState } from "./store/editorState";
import { initialState, previewKey } from "./store/editorState";
import { listenWindowBus } from "./windowBus";
import type { WindowBusEvent } from "./windowBusTypes";
import { canReadProjectFileContent } from "../features/files/fileContentRules";

function isRuntimeMod(mod: EditorModSummaryDto): boolean {
  const id = mod.id.toLowerCase();
  return id === "core" || id === "core-game" || id.startsWith("core-");
}

function selectStartupMod(mods: EditorModSummaryDto[]): EditorModSummaryDto | undefined {
  return (
    mods.find((mod) => mod.visibleSceneCount > 0 && mod.status === "valid" && !isRuntimeMod(mod)) ??
    mods.find((mod) => mod.visibleSceneCount > 0 && !isRuntimeMod(mod)) ??
    mods.find((mod) => mod.visibleSceneCount > 0) ??
    mods[0]
  );
}

function projectFileContentKey(modId: string, relativePath: string): string {
  return `${modId}:${relativePath}`;
}

function canReadProjectFile(file: EditorProjectFileDto): boolean {
  return canReadProjectFileContent(file);
}

interface EditorStoreValue {
  state: EditorState;
  scanMods: () => Promise<void>;
  selectMod: (modId: string) => Promise<void>;
  loadProjectTree: (modId: string) => Promise<void>;
  refreshProjectTree: (modId: string) => Promise<void>;
  loadEditorSession: (sessionId: string) => Promise<void>;
  selectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity: (entityId: string) => void;
  selectAsset: (asset: ManagedAssetDto | null) => void;
  selectProjectFile: (file: EditorProjectFileDto) => void;
  selectWorkspaceTab: (tabId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  openComponent: (componentId: string, context?: Record<string, string>) => void;
  focusComponent: (instanceId: string, componentId: string) => void;
  moveComponent: (instanceId: string, placement: string) => void;
  closeComponent: (instanceId: string, componentId: string) => void;
  revealSelectedProjectFile: () => Promise<void>;
  createExpectedFolder: (expectedPath: string) => Promise<void>;
  loadSceneHierarchy: (modId: string, sceneId: string) => Promise<void>;
  regeneratePreview: (modId: string, sceneId: string, forceRegenerate?: boolean) => Promise<void>;
  validateSelectedMod: () => Promise<void>;
  revealSelectedModFolder: () => Promise<void>;
  revealSelectedSceneDocument: () => Promise<void>;
  openSelectedMod: () => Promise<void>;
  recordEvent: (event: EditorEvent) => void;
  returnToStartup: () => Promise<void>;
  toggleInspectorSection: (sectionId: string) => void;
  setPreviewPlaying: (playing: boolean) => void;
  setContentFilter: (filter: string | null) => void;
  setFileDirty: (path: string, dirty: boolean) => void;
}

const EditorStoreContext = createContext<EditorStoreValue | null>(null);

export function EditorStoreProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);

  const emit = useCallback((event: EditorEvent) => {
    dispatch({ type: "event", event });
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenPreviewProgress(
      (payload) => {
        if (cancelled) {
          return;
        }
        const progress = payload.total > 0 ? payload.current / payload.total : 0;
        dispatch({ type: "taskProgress", taskId: `preview:${payload.modId}:${payload.sceneId}`, progress });
        emit({
          type: "ScenePreviewFrameGenerated",
          modId: payload.modId,
          sceneId: payload.sceneId,
          current: payload.current,
          total: payload.total,
        });
      },
    ).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [emit]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenWindowBus((event) => {
      if (!cancelled) {
        dispatch({ type: "windowEvent", event });
      }
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const regeneratePreview = useCallback(
    async (modId: string, sceneId: string, forceRegenerate = false) => {
      emit({ type: "ScenePreviewRequested", modId, sceneId });
      const taskId = `preview:${modId}:${sceneId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Rendering ${sceneId}`, "local", "ScenePreviewWorkspace") });
      emit({ type: "ScenePreviewStarted", modId, sceneId });

      try {
        const preview = await requestScenePreview(modId, sceneId, forceRegenerate);
        dispatch({ type: "previewLoaded", preview });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "ScenePreviewCompleted", modId, sceneId });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "ScenePreviewFailed", modId, sceneId, error: message });
      }
    },
    [emit],
  );

  const loadSceneHierarchy = useCallback(
    async (modId: string, sceneId: string) => {
      if (state.sceneHierarchies[previewKey(modId, sceneId)]) {
        return;
      }

      emit({ type: "SceneHierarchyRequested", modId, sceneId });
      const taskId = `scene-hierarchy:${modId}:${sceneId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Indexing ${sceneId}`, "local", "SceneHierarchy") });

      try {
        const hierarchy = await getSceneHierarchy(modId, sceneId);
        dispatch({ type: "sceneHierarchyLoaded", hierarchy });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "SceneHierarchyLoaded", modId, sceneId, entityCount: hierarchy.entityCount });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "SceneHierarchyFailed", modId, sceneId, error: message });
      }
    },
    [emit, state.sceneHierarchies],
  );

  const loadProjectTree = useCallback(
    async (modId: string, force = false) => {
      if (!force && state.projectTrees[modId]) {
        return;
      }

      emit({ type: "ProjectTreeRequested", modId });
      const taskId = `project-tree:${modId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Indexing files ${modId}`, "local", "ProjectExplorer") });

      try {
        const [tree, structureTree] = await Promise.all([
          getProjectTree(modId),
          getProjectStructureTree(modId),
        ]);
        dispatch({ type: "projectTreeLoaded", tree });
        dispatch({ type: "projectStructureTreeLoaded", tree: structureTree });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "ProjectTreeLoaded", modId, fileCount: tree.totalFiles });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "ProjectTreeFailed", modId, error: message });
      }
    },
    [emit, state.projectTrees],
  );

  const selectScene = useCallback(
    async (scene: EditorSceneSummaryDto) => {
      const modId = selectedModId(state.selection);
      if (!modId) return;
      dispatch({ type: "selectionChanged", selection: { kind: "scene", modId, sceneId: scene.id } });
      emit({ type: "SceneSelected", modId, sceneId: scene.id });

      if (!state.previews[previewKey(modId, scene.id)]) {
          await regeneratePreview(modId, scene.id, false);
      }
      await loadSceneHierarchy(modId, scene.id);
    },
    [emit, loadSceneHierarchy, regeneratePreview, state.previews, state.selection],
  );

  const selectSceneEntity = useCallback(
    (entityId: string) => {
      const modId = selectedModId(state.selection);
      const sceneId = selectedSceneId(state.selection);
      if (!modId || !sceneId) return;
      dispatch({ type: "selectionChanged", selection: { kind: "entity", modId, sceneId, entityId } });
      emit({ type: "InspectorContextChanged", contextKind: "entity", id: entityId });
    },
    [emit, state.selection],
  );

  const selectAsset = useCallback(
    (asset: ManagedAssetDto | null) => {
      if (asset) {
        const modId = selectedModId(state.selection) ?? state.modDetails?.id ?? asset.assetKey.split("/")[0];
        dispatch({
          type: "selectionChanged",
          selection: {
            kind: "asset",
            modId,
            assetKey: asset.assetKey,
            filePath: asset.descriptorRelativePath,
          },
        });
        emit({ type: "AssetSelected", modId, assetKey: asset.assetKey, kind: asset.kind });
        emit({ type: "InspectorContextChanged", contextKind: "asset", id: asset.assetKey });
      }
    },
    [emit, state.modDetails?.id, state.selection],
  );

  const selectProjectFile = useCallback(
    (file: EditorProjectFileDto) => {
      if (file.isDir) {
        return;
      }

      const modId = selectedModId(state.selection) ?? state.modDetails?.id;
      if (modId) {
        dispatch({ type: "selectionChanged", selection: { kind: "projectFile", modId, path: file.relativePath } });
      }
      if (modId) {
        emit({ type: "ProjectFileSelected", modId, path: file.relativePath, kind: file.kind });
        if (canReadProjectFile(file) && !state.projectFileContents[projectFileContentKey(modId, file.relativePath)]) {
          emit({ type: "ProjectFileReadRequested", modId, path: file.relativePath });
          const taskId = `read-project-file:${modId}:${file.relativePath}`;
          dispatch({ type: "taskStarted", task: createTask(taskId, `Reading ${file.name}`, "local", "CenterWorkspace") });
          void readProjectFile(modId, file.relativePath)
            .then((content) => {
              dispatch({ type: "projectFileContentLoaded", content });
              dispatch({ type: "taskFinished", taskId });
              emit({ type: "ProjectFileReadCompleted", modId, path: file.relativePath });
            })
            .catch((error) => {
              const message = error instanceof Error ? error.message : String(error);
              dispatch({ type: "taskFailed", taskId, error: message });
              emit({ type: "ProjectFileReadFailed", modId, path: file.relativePath, error: message });
            });
        }
      }
      emit({ type: "WorkspaceTabOpened", tabId: `file:${file.relativePath}`, resourcePath: file.relativePath });
      emit({ type: "InspectorContextChanged", contextKind: file.kind === "texture" || file.kind === "spritesheet" ? "asset" : "file", id: file.relativePath });
    },
    [emit, state.modDetails?.id, state.projectFileContents, state.selection],
  );

  const selectWorkspaceTab = useCallback(
    (tabId: string) => {
      dispatch({ type: "workspaceTabSelected", tabId });
      emit({ type: "WorkspaceTabSelected", tabId });
    },
    [emit],
  );

  const closeWorkspaceTab = useCallback(
    (tabId: string) => {
      dispatch({ type: "workspaceTabClosed", tabId });
      emit({ type: "WorkspaceTabClosed", tabId });
    },
    [emit],
  );

  const loadModDetails = useCallback(
    async (modId: string, preferredSceneId?: string | null) => {
      emit({ type: "ModDetailsRequested", modId });
      const taskId = `load-mod-details:${modId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Loading ${modId}`, "local", "ModInspectorPanel") });

      try {
        const details = await getModDetails(modId);
        dispatch({ type: "modDetailsLoaded", details });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "ModDetailsLoaded", modId: details.id });
        await loadProjectTree(details.id);
        const firstScene =
          details.scenes.find((scene) => scene.id === preferredSceneId) ??
          details.scenes.find((scene) => scene.launcherVisible) ??
          details.scenes[0];
        if (firstScene) {
          dispatch({ type: "selectionChanged", selection: { kind: "scene", modId: details.id, sceneId: firstScene.id } });
          emit({ type: "SceneSelected", modId: details.id, sceneId: firstScene.id });
          await regeneratePreview(details.id, firstScene.id, false);
          await loadSceneHierarchy(details.id, firstScene.id);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "ModDetailsFailed", modId, error: message });
      }
    },
    [emit, loadProjectTree, loadSceneHierarchy, regeneratePreview],
  );

  const loadEditorSession = useCallback(
    async (sessionId: string) => {
      const taskId = `load-editor-session:${sessionId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, "Loading editor session", "blocking", "MainEditorWindow") });

      try {
        const session = await getEditorSession(sessionId);
        dispatch({
          type: "sessionOpened",
          session: {
            modId: session.modId,
            rootPath: session.rootPath,
            sessionId: session.sessionId,
            createdAt: session.createdAt,
            selectedSceneId: session.selectedSceneId,
          },
        });
        dispatch({ type: "modSelected", modId: session.modId });
        emit({ type: "EditorSessionLoaded", modId: session.modId, sessionId: session.sessionId });
        emit({ type: "DockLayoutLoaded", layoutId: "default" });
        await loadModDetails(session.modId, session.selectedSceneId);
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "WorkspaceReady", sessionId: session.sessionId });
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "OpenModFailed", modId: "unknown", error: message });
      }
    },
    [emit, loadModDetails],
  );

  const selectMod = useCallback(
    async (modId: string) => {
      dispatch({ type: "modSelected", modId });
      emit({ type: "ModSelected", modId });
      await loadModDetails(modId);
    },
    [emit, loadModDetails],
  );

  const scanMods = useCallback(async () => {
    const mods = await runEditorTask({
      completed: (result) => ({ type: "ModsScanCompleted", modCount: result.length }),
      dispatch,
      emit,
      failed: (error) => ({ type: "ModsScanFailed", error }),
      requested: { type: "ModsScanRequested" },
      run: listKnownMods,
      started: { type: "ModsScanStarted" },
      task: createTask("scan-mods", "Scanning mods", "background", "ModsPanel"),
    });

    if (!mods) {
      return;
    }

    dispatch({ type: "modsLoaded", mods });
    const startupMod = selectStartupMod(mods);
    if (startupMod) {
      await selectMod(startupMod.id);
    }
  }, [emit, selectMod]);

  const openSelectedMod = useCallback(async () => {
    const modId = selectedModId(state.selection);
    if (!modId) return;
    const result = await runEditorTask({
      completed: (opened) => ({
        type: "OpenModCompleted",
        modId: opened.modId,
        sessionId: opened.sessionId,
      }),
      dispatch,
      emit,
      failed: (error) => ({ type: "OpenModFailed", modId, error }),
      requested: { type: "OpenModRequested", modId },
      run: () => openModWorkspace(modId, selectedSceneId(state.selection)),
      task: createTask(`open-mod:${modId}`, `Opening ${modId}`, "blocking", "StartupDialog"),
    });

    if (result) {
      emit({ type: "MainEditorWindowRequested", modId: result.modId, sessionId: result.sessionId });
    }
  }, [emit, state.selection]);

  const validateSelectedMod = useCallback(async () => {
    const modId = selectedModId(state.selection);
    if (!modId) return;

    const details = await runEditorTask({
      completed: () => ({ type: "ModValidationCompleted", modId }),
      dispatch,
      emit,
      failed: (error) => ({ type: "ModValidationFailed", modId, error }),
      requested: { type: "ModValidationRequested", modId },
      run: () => validateMod(modId),
      task: createTask(`validate-mod:${modId}`, `Validating ${modId}`, "local", "ModInspectorPanel"),
    });

    if (details) {
      dispatch({ type: "modDetailsLoaded", details });
    }
  }, [emit, state.selection]);

  const revealSelectedModFolder = useCallback(async () => {
    const modId = selectedModId(state.selection);
    if (!modId) return;
    emit({ type: "RevealPathRequested", pathKind: "mod", modId });
    try {
      const path = await revealModFolder(modId);
      emit({ type: "RevealPathCompleted", pathKind: "mod", path });
    } catch (error) {
      emit({ type: "RevealPathFailed", pathKind: "mod", error: error instanceof Error ? error.message : String(error) });
    }
  }, [emit, state.selection]);

  const revealSelectedSceneDocument = useCallback(async () => {
    const modId = selectedModId(state.selection);
    const sceneId = selectedSceneId(state.selection);
    if (!modId || !sceneId) return;
    emit({ type: "RevealPathRequested", pathKind: "scene", modId, sceneId });
    try {
      const path = await revealSceneDocument(modId, sceneId);
      emit({ type: "RevealPathCompleted", pathKind: "scene", path });
    } catch (error) {
      emit({ type: "RevealPathFailed", pathKind: "scene", error: error instanceof Error ? error.message : String(error) });
    }
  }, [emit, state.selection]);

  const revealSelectedProjectFile = useCallback(async () => {
    const modId = selectedModId(state.selection) ?? state.modDetails?.id;
    const relativePath = selectedFilePath(state.selection);
    if (!modId || !relativePath) return;
    emit({ type: "ProjectFileRevealRequested", modId, path: relativePath });
    try {
      const path = await revealProjectFile(modId, relativePath);
      emit({ type: "ProjectFileRevealCompleted", modId, path });
    } catch (error) {
      emit({
        type: "ProjectFileRevealFailed",
        modId,
        path: relativePath,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, [emit, state.modDetails?.id, state.selection]);

  const createExpectedFolder = useCallback(async (expectedPath: string) => {
    const modId = selectedModId(state.selection) ?? state.modDetails?.id;
    if (!modId) return;
    emit({ type: "ExpectedProjectFolderCreateRequested", modId, expectedPath });
    try {
      const path = await createExpectedProjectFolder(modId, expectedPath);
      emit({ type: "ExpectedProjectFolderCreateCompleted", modId, path });
      const [tree, structureTree] = await Promise.all([
        getProjectTree(modId),
        getProjectStructureTree(modId),
      ]);
      dispatch({ type: "projectTreeLoaded", tree });
      dispatch({ type: "projectStructureTreeLoaded", tree: structureTree });
    } catch (error) {
      emit({
        type: "ExpectedProjectFolderCreateFailed",
        modId,
        expectedPath,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }, [emit, state.modDetails?.id, state.selection]);

  const value = useMemo<EditorStoreValue>(
    () => ({
      state,
      scanMods,
      selectMod,
      loadProjectTree,
      refreshProjectTree: async (modId) => {
        await loadProjectTree(modId, true);
      },
      loadEditorSession,
      selectScene,
      selectSceneEntity,
      selectProjectFile,
      selectWorkspaceTab,
      closeWorkspaceTab,
      openComponent: (componentId, context) => {
        emit({ type: "ComponentOpenRequested", componentId, context });
        emit({ type: "ComponentOpened", instanceId: `${componentId}:singleton`, componentId });
      },
      focusComponent: (instanceId, componentId) => {
        emit({ type: "ComponentFocused", instanceId, componentId });
      },
      moveComponent: (instanceId, placement) => {
        emit({ type: "ComponentMoved", instanceId, placement });
      },
      closeComponent: (instanceId, componentId) => {
        emit({ type: "ComponentClosed", instanceId, componentId });
      },
      loadSceneHierarchy,
      regeneratePreview,
      validateSelectedMod,
      selectAsset,
      revealSelectedModFolder,
      revealSelectedSceneDocument,
      revealSelectedProjectFile,
      createExpectedFolder,
      openSelectedMod,
      recordEvent: emit,
      returnToStartup: async () => {
        const sessionId = state.activeSession?.sessionId;
        if (sessionId) {
          try {
            await closeEditorSession(sessionId);
          } catch {
            // Returning to the launcher should not be blocked by a stale backend session.
          }
          emit({ type: "EditorSessionClosed", sessionId });
        }
        dispatch({ type: "returnToStartup" });
      },
      toggleInspectorSection: (sectionId) => {
        dispatch({ type: "toggleInspectorSection", sectionId });
        emit({ type: "InspectorSectionToggled", sectionId });
      },
      setPreviewPlaying: (playing) => {
        dispatch({ type: "setPreviewPlaying", playing });
        emit({ type: "PreviewPlaybackToggled", playing });
      },
      setContentFilter: (filter) => {
        dispatch({ type: "setContentFilter", filter });
        emit({ type: "ContentFilterChanged", filter });
      },
      setFileDirty: (path, dirty) => {
        dispatch({ type: "setFileDirty", path, dirty });
        emit({ type: "FileDirtyStateChanged", path, dirty });
      },
    }),
    [closeWorkspaceTab, createExpectedFolder, emit, loadEditorSession, loadProjectTree, loadSceneHierarchy, openSelectedMod, regeneratePreview, revealSelectedModFolder, revealSelectedProjectFile, revealSelectedSceneDocument, scanMods, selectAsset, selectMod, selectProjectFile, selectScene, selectSceneEntity, selectWorkspaceTab, state, validateSelectedMod],
  );

  return <EditorStoreContext.Provider value={value}>{children}</EditorStoreContext.Provider>;
}

export function useEditorStore(): EditorStoreValue {
  const value = useContext(EditorStoreContext);
  if (!value) {
    throw new Error("useEditorStore must be used inside EditorStoreProvider");
  }
  return value;
}
