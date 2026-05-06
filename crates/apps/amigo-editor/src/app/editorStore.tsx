import React, { createContext, useCallback, useContext, useEffect, useMemo, useReducer, useRef } from "react";
import { closeEditorSession, createExpectedProjectFolder, createModProject as createModProjectApi, deleteModProject as deleteModProjectApi, getEditorSession, getModDetails, getProjectStructureTree, getProjectTree, getSceneHierarchy, listKnownMods, openModWorkspace, readProjectFile, requestScenePreview, revealModFolder, revealProjectFile, revealSceneDocument, validateMod } from "../api/editorApi";
import type { CreateModProjectRequestDto, EditorModDetailsDto, EditorModSummaryDto, EditorProjectFileContentDto, EditorProjectFileDto, EditorProjectStructureTreeDto, EditorProjectTreeDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, ManagedAssetDto, OpenModResultDto, ScenePreviewDto } from "../api/dto";
import type { EditorEvent } from "./editorEvents";
import type { EditorTask } from "./editorTasks";
import { createTask, failTask, finishTask } from "./editorTasks";
import { listenPreviewProgress } from "./previewProgressBus";
import { runEditorTask } from "./runEditorTask";
import { selectedFilePath, selectedModId, selectedSceneId } from "./selectionSelectors";
import type { Action } from "./store/editorActions";
import { reducer } from "./store/editorReducer";
import { initialState, previewKey } from "./store/editorState";
import { useSelectionActions } from "./store/hooks/useSelectionActions";
import type { EditorStoreValue } from "./store/storeActionTypes";
import { listenWindowBus } from "./windowBus";
import type { WindowBusEvent } from "./windowBusTypes";
import { canReadProjectFileContent } from "../features/files/fileContentRules";
import { projectFileFromManagedAsset } from "../assets/assetProjectFiles";

function isRuntimeMod(mod: EditorModSummaryDto): boolean {
  const id = mod.id.toLowerCase();
  return id === "core" || id === "core-game" || id.startsWith("core-");
}

function isHiddenStartupProject(mod: EditorModSummaryDto): boolean {
  const id = mod.id.toLowerCase();
  const name = mod.name.toLowerCase();
  return (
    id === "ink-wars" ||
    name.includes("ink wars") ||
    id.includes("core-runtime") ||
    name.includes("core runtime") ||
    id.includes("dev-tools") ||
    name.includes("dev tools")
  );
}

function selectStartupMod(mods: EditorModSummaryDto[]): EditorModSummaryDto | undefined {
  const visibleMods = mods.filter((mod) => !isHiddenStartupProject(mod));
  return (
    visibleMods.find((mod) => mod.visibleSceneCount > 0 && mod.status === "valid" && !isRuntimeMod(mod)) ??
    visibleMods.find((mod) => mod.visibleSceneCount > 0 && !isRuntimeMod(mod)) ??
    visibleMods.find((mod) => mod.visibleSceneCount > 0) ??
    visibleMods[0]
  );
}

function projectFileContentKey(modId: string, relativePath: string): string {
  return `${modId}:${relativePath}`;
}

function canReadProjectFile(file: EditorProjectFileDto): boolean {
  return canReadProjectFileContent(file);
}

const EditorStoreContext = createContext<EditorStoreValue | null>(null);

export function EditorStoreProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const selectedModRef = useRef<string | null>(selectedModId(initialState.selection));

  const emit = useCallback((event: EditorEvent) => {
    dispatch({ type: "event", event });
  }, []);

  useEffect(() => {
    selectedModRef.current = selectedModId(state.selection);
  }, [state.selection]);

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
    async (modId: string, sceneId: string, force = false) => {
      if (!force && state.sceneHierarchies[previewKey(modId, sceneId)]) {
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

  const { selectSceneEntity, selectUiNode } = useSelectionActions({
    dispatch,
    emit,
    selection: state.selection,
  });

  const selectAsset = useCallback(
    (asset: ManagedAssetDto | null) => {
      if (asset) {
        const modId = selectedModId(state.selection) ?? state.modDetails?.id ?? asset.assetKey.split("/")[0];
        const file = projectFileFromManagedAsset(asset);
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
        emit({ type: "WorkspaceTabOpened", tabId: `file:${file.relativePath}`, resourcePath: file.relativePath });
        emit({ type: "InspectorContextChanged", contextKind: "asset", id: asset.assetKey });
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
    },
    [emit, state.modDetails?.id, state.projectFileContents, state.selection],
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
        if (selectedModRef.current !== modId) {
          dispatch({ type: "taskFinished", taskId });
          return;
        }
        dispatch({ type: "modDetailsLoaded", details });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "ModDetailsLoaded", modId: details.id });
        await loadProjectTree(details.id);
        if (selectedModRef.current !== details.id) {
          return;
        }
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
        if (selectedModRef.current !== modId) {
          dispatch({ type: "taskFinished", taskId });
          return;
        }
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
      selectedModRef.current = modId;
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
    const preservedModId = selectedModRef.current;
    if (preservedModId && mods.some((mod) => mod.id === preservedModId)) {
      return;
    }

    const startupMod = selectStartupMod(mods);
    if (startupMod) {
      await selectMod(startupMod.id);
    }
  }, [emit, selectMod]);

  const createModProject = useCallback(async (request: CreateModProjectRequestDto) => {
    const taskId = `create-mod-project:${request.projectId}`;
    emit({ type: "ModProjectCreateRequested", projectId: request.projectId, projectName: request.projectName, projectType: request.projectType });
    dispatch({ type: "taskStarted", task: createTask(taskId, `Creating ${request.projectName}`, "blocking", "StartupDialog") });
    emit({ type: "ModProjectCreateStarted", projectId: request.projectId });
    try {
      const result = await createModProjectApi(request);
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "ModProjectCreateCompleted", modId: result.modId, rootPath: result.rootPath });
      selectedModRef.current = result.modId;
      await scanMods();
      await selectMod(result.modId);
      return result;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "ModProjectCreateFailed", projectId: request.projectId, error: message });
      throw error;
    }
  }, [emit, scanMods, selectMod]);

  const deleteModProject = useCallback(async (modId: string) => {
    const taskId = `delete-mod-project:${modId}`;
    emit({ type: "ModProjectDeleteRequested", modId });
    dispatch({ type: "taskStarted", task: createTask(taskId, `Deleting ${modId}`, "blocking", "StartupDialog") });
    emit({ type: "ModProjectDeleteStarted", modId });
    try {
      const deletedPath = await deleteModProjectApi(modId);
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "ModProjectDeleteCompleted", modId, path: deletedPath });
      if (selectedModRef.current === modId) {
        selectedModRef.current = null;
      }
      await scanMods();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "ModProjectDeleteFailed", modId, error: message });
      throw error;
    }
  }, [emit, scanMods]);

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
      createModProject,
      deleteModProject,
      selectMod,
      loadProjectTree,
      refreshProjectTree: async (modId) => {
        await loadProjectTree(modId, true);
      },
      loadEditorSession,
      selectScene,
      selectSceneEntity,
      selectUiNode,
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
    [closeWorkspaceTab, createExpectedFolder, createModProject, deleteModProject, emit, loadEditorSession, loadProjectTree, loadSceneHierarchy, openSelectedMod, regeneratePreview, revealSelectedModFolder, revealSelectedProjectFile, revealSelectedSceneDocument, scanMods, selectAsset, selectMod, selectProjectFile, selectScene, selectSceneEntity, selectUiNode, selectWorkspaceTab, state, validateSelectedMod],
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
