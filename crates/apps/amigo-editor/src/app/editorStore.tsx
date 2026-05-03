import React, { createContext, useCallback, useContext, useEffect, useMemo, useReducer } from "react";
import { closeEditorSession, getEditorSession, getModDetails, getProjectTree, getSceneHierarchy, listKnownMods, openModWorkspace, readProjectFile, requestScenePreview, revealModFolder, revealProjectFile, revealSceneDocument, validateMod } from "../api/editorApi";
import type { EditorModDetailsDto, EditorModSummaryDto, EditorProjectFileContentDto, EditorProjectFileDto, EditorProjectTreeDto, EditorSceneHierarchyDto, EditorSceneSummaryDto, OpenModResultDto, ScenePreviewDto } from "../api/dto";
import type { EditorEvent } from "./editorEvents";
import type { EditorTask } from "./editorTasks";
import { createTask, failTask, finishTask } from "./editorTasks";
import { listenPreviewProgress } from "./previewProgressBus";
import { listenWindowBus } from "./windowBus";
import type { WindowBusEvent } from "./windowBusTypes";

interface EditorState {
  appMode: "startup" | "editor";
  activeSession: OpenModResultDto | null;
  mods: EditorModSummaryDto[];
  selectedModId: string | null;
  selectedSceneId: string | null;
  selectedEntityId: string | null;
  selectedFilePath: string | null;
  activeWorkspaceTabId: string;
  openedFilePaths: string[];
  modDetails: EditorModDetailsDto | null;
  projectTrees: Record<string, EditorProjectTreeDto>;
  projectFileContents: Record<string, EditorProjectFileContentDto>;
  previews: Record<string, ScenePreviewDto>;
  sceneHierarchies: Record<string, EditorSceneHierarchyDto>;
  tasks: Record<string, EditorTask>;
  events: EditorEvent[];
  windowEvents: WindowBusEvent[];
  previewPlaying: boolean;
  contentFilter: string | null;
  openInspectorSections: Record<string, boolean>;
  dirtyFiles: Record<string, boolean>;
  hasDirtyState: boolean;
}

const initialState: EditorState = {
  appMode: "startup",
  activeSession: null,
  mods: [],
  selectedModId: null,
  selectedSceneId: null,
  selectedEntityId: null,
  selectedFilePath: null,
  activeWorkspaceTabId: "scene-preview",
  openedFilePaths: [],
  modDetails: null,
  projectTrees: {},
  projectFileContents: {},
  previews: {},
  sceneHierarchies: {},
  tasks: {},
  events: [],
  windowEvents: [],
  previewPlaying: true,
  contentFilter: null,
  openInspectorSections: {
    summary: true,
    content: true,
    scene: true,
    dependencies: false,
    capabilities: false,
    diagnostics: true,
    events: false,
  },
  dirtyFiles: {},
  hasDirtyState: false,
};

type Action =
  | { type: "event"; event: EditorEvent }
  | { type: "windowEvent"; event: WindowBusEvent }
  | { type: "setFileDirty"; path: string; dirty: boolean }
  | { type: "modsLoaded"; mods: EditorModSummaryDto[] }
  | { type: "modSelected"; modId: string }
  | { type: "modDetailsLoaded"; details: EditorModDetailsDto }
  | { type: "projectTreeLoaded"; tree: EditorProjectTreeDto }
  | { type: "sceneSelected"; sceneId: string }
  | { type: "sceneEntitySelected"; entityId: string }
  | { type: "projectFileSelected"; file: EditorProjectFileDto }
  | { type: "projectFileContentLoaded"; content: EditorProjectFileContentDto }
  | { type: "workspaceTabSelected"; tabId: string }
  | { type: "workspaceTabClosed"; tabId: string }
  | { type: "previewLoaded"; preview: ScenePreviewDto }
  | { type: "sceneHierarchyLoaded"; hierarchy: EditorSceneHierarchyDto }
  | { type: "taskStarted"; task: EditorTask }
  | { type: "taskFinished"; taskId: string }
  | { type: "taskFailed"; taskId: string; error: string }
  | { type: "taskProgress"; taskId: string; progress: number }
  | { type: "sessionOpened"; session: OpenModResultDto }
  | { type: "returnToStartup" }
  | { type: "toggleInspectorSection"; sectionId: string }
  | { type: "setPreviewPlaying"; playing: boolean }
  | { type: "setContentFilter"; filter: string | null };

function previewKey(modId: string, sceneId: string): string {
  return `${modId}:${sceneId}`;
}

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
  return ["manifest", "sceneDocument", "script", "yaml"].includes(file.kind);
}

function reducer(state: EditorState, action: Action): EditorState {
  switch (action.type) {
    case "event":
      return { ...state, events: [action.event, ...state.events].slice(0, 80) };
    case "windowEvent":
      if (
        action.event.type === "CacheInvalidated" &&
        (!action.event.payload.projectCacheId || action.event.payload.projectCacheId === state.modDetails?.projectCacheId)
      ) {
        if (action.event.payload.modId && action.event.payload.sceneId) {
          const nextPreviews = { ...state.previews };
          delete nextPreviews[previewKey(action.event.payload.modId, action.event.payload.sceneId)];
          return {
            ...state,
            previews: nextPreviews,
            windowEvents: [action.event, ...state.windowEvents].slice(0, 120),
          };
        }
        return {
          ...state,
          previews: {},
          windowEvents: [action.event, ...state.windowEvents].slice(0, 120),
        };
      }
      return { ...state, windowEvents: [action.event, ...state.windowEvents].slice(0, 120) };
    case "setFileDirty": {
      const dirtyFiles = { ...state.dirtyFiles };
      if (action.dirty) {
        dirtyFiles[action.path] = true;
      } else {
        delete dirtyFiles[action.path];
      }
      return { ...state, dirtyFiles, hasDirtyState: Object.keys(dirtyFiles).length > 0 };
    }
    case "modsLoaded":
      return { ...state, mods: action.mods };
    case "modSelected":
      return {
        ...state,
        selectedModId: action.modId,
        selectedSceneId: null,
        selectedEntityId: null,
        selectedFilePath: null,
        activeWorkspaceTabId: "scene-preview",
        openedFilePaths: [],
        modDetails: null,
      };
    case "modDetailsLoaded": {
      const firstScene = action.details.scenes.find((scene) => scene.launcherVisible)?.id ?? action.details.scenes[0]?.id ?? null;
      return { ...state, modDetails: action.details, selectedSceneId: state.selectedSceneId ?? firstScene };
    }
    case "projectTreeLoaded":
      return {
        ...state,
        projectTrees: {
          ...state.projectTrees,
          [action.tree.modId]: action.tree,
        },
      };
    case "sceneSelected":
      return { ...state, selectedSceneId: action.sceneId, selectedEntityId: null, activeWorkspaceTabId: "scene-preview" };
    case "sceneEntitySelected":
      return { ...state, selectedEntityId: action.entityId };
    case "projectFileSelected":
      return {
        ...state,
        selectedFilePath: action.file.relativePath,
        activeWorkspaceTabId: `file:${action.file.relativePath}`,
        openedFilePaths: state.openedFilePaths.includes(action.file.relativePath)
          ? state.openedFilePaths
          : [...state.openedFilePaths, action.file.relativePath],
      };
    case "projectFileContentLoaded":
      return {
        ...state,
        projectFileContents: {
          ...state.projectFileContents,
          [`${action.content.modId}:${action.content.relativePath}`]: action.content,
        },
      };
    case "workspaceTabSelected":
      return { ...state, activeWorkspaceTabId: action.tabId };
    case "workspaceTabClosed": {
      if (!action.tabId.startsWith("file:")) {
        return state;
      }
      const relativePath = action.tabId.slice("file:".length);
      const openedFilePaths = state.openedFilePaths.filter((path) => path !== relativePath);
      const activeWorkspaceTabId = state.activeWorkspaceTabId === action.tabId ? "scene-preview" : state.activeWorkspaceTabId;
      return {
        ...state,
        openedFilePaths,
        selectedFilePath: state.selectedFilePath === relativePath ? null : state.selectedFilePath,
        activeWorkspaceTabId,
      };
    }
    case "previewLoaded":
      return { ...state, previews: { ...state.previews, [previewKey(action.preview.modId, action.preview.sceneId)]: action.preview } };
    case "sceneHierarchyLoaded":
      return {
        ...state,
        selectedEntityId:
          action.hierarchy.sceneId === state.selectedSceneId
            ? action.hierarchy.entities.find((entity) => entity.id === state.selectedEntityId)?.id ??
              action.hierarchy.entities[0]?.id ??
              null
            : state.selectedEntityId,
        sceneHierarchies: {
          ...state.sceneHierarchies,
          [previewKey(action.hierarchy.modId, action.hierarchy.sceneId)]: action.hierarchy,
        },
      };
    case "taskStarted":
      return { ...state, tasks: { ...state.tasks, [action.task.id]: action.task } };
    case "taskFinished": {
      const task = state.tasks[action.taskId];
      return task ? { ...state, tasks: { ...state.tasks, [action.taskId]: finishTask(task) } } : state;
    }
    case "taskFailed": {
      const task = state.tasks[action.taskId];
      return task ? { ...state, tasks: { ...state.tasks, [action.taskId]: failTask(task, action.error) } } : state;
    }
    case "taskProgress": {
      const task = state.tasks[action.taskId];
      return task ? { ...state, tasks: { ...state.tasks, [action.taskId]: { ...task, progress: action.progress } } } : state;
    }
    case "sessionOpened":
      return { ...state, appMode: "editor", activeSession: action.session };
    case "returnToStartup":
      return { ...state, appMode: "startup", activeSession: null };
    case "toggleInspectorSection":
      return { ...state, openInspectorSections: { ...state.openInspectorSections, [action.sectionId]: !state.openInspectorSections[action.sectionId] } };
    case "setPreviewPlaying":
      return { ...state, previewPlaying: action.playing };
    case "setContentFilter":
      return { ...state, contentFilter: action.filter };
    default:
      return state;
  }
}

interface EditorStoreValue {
  state: EditorState;
  scanMods: () => Promise<void>;
  selectMod: (modId: string) => Promise<void>;
  loadProjectTree: (modId: string) => Promise<void>;
  loadEditorSession: (sessionId: string) => Promise<void>;
  selectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
  selectSceneEntity: (entityId: string) => void;
  selectProjectFile: (file: EditorProjectFileDto) => void;
  selectWorkspaceTab: (tabId: string) => void;
  closeWorkspaceTab: (tabId: string) => void;
  revealSelectedProjectFile: () => Promise<void>;
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
    async (modId: string) => {
      if (state.projectTrees[modId]) {
        return;
      }

      emit({ type: "ProjectTreeRequested", modId });
      const taskId = `project-tree:${modId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Indexing files ${modId}`, "local", "ProjectExplorer") });

      try {
        const tree = await getProjectTree(modId);
        dispatch({ type: "projectTreeLoaded", tree });
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
      if (!state.selectedModId) return;
      const modId = state.selectedModId;
      dispatch({ type: "sceneSelected", sceneId: scene.id });
      emit({ type: "SceneSelected", modId, sceneId: scene.id });

      if (!state.previews[previewKey(modId, scene.id)]) {
          await regeneratePreview(modId, scene.id, false);
      }
      await loadSceneHierarchy(modId, scene.id);
    },
    [emit, loadSceneHierarchy, regeneratePreview, state.previews, state.selectedModId],
  );

  const selectSceneEntity = useCallback(
    (entityId: string) => {
      dispatch({ type: "sceneEntitySelected", entityId });
      emit({ type: "InspectorContextChanged", contextKind: "entity", id: entityId });
    },
    [emit],
  );

  const selectProjectFile = useCallback(
    (file: EditorProjectFileDto) => {
      if (file.isDir) {
        return;
      }

      const modId = state.selectedModId ?? state.modDetails?.id;
      dispatch({ type: "projectFileSelected", file });
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
    [emit, state.modDetails?.id, state.projectFileContents, state.selectedModId],
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
          dispatch({ type: "sceneSelected", sceneId: firstScene.id });
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
    emit({ type: "ModsScanRequested" });
    const taskId = "scan-mods";
    dispatch({ type: "taskStarted", task: createTask(taskId, "Scanning mods", "background", "ModsPanel") });
    emit({ type: "ModsScanStarted" });

    try {
      const mods = await listKnownMods();
      dispatch({ type: "modsLoaded", mods });
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "ModsScanCompleted", modCount: mods.length });
      const startupMod = selectStartupMod(mods);
      if (startupMod) {
        await selectMod(startupMod.id);
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "ModsScanFailed", error: message });
    }
  }, [emit, selectMod]);

  const openSelectedMod = useCallback(async () => {
    if (!state.selectedModId) return;
    const modId = state.selectedModId;
    emit({ type: "OpenModRequested", modId });
    const taskId = `open-mod:${modId}`;
    dispatch({ type: "taskStarted", task: createTask(taskId, `Opening ${modId}`, "blocking", "StartupDialog") });

    try {
      const result = await openModWorkspace(modId, state.selectedSceneId);
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "OpenModCompleted", modId: result.modId, sessionId: result.sessionId });
      emit({ type: "MainEditorWindowRequested", modId: result.modId, sessionId: result.sessionId });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "OpenModFailed", modId, error: message });
    }
  }, [emit, state.selectedModId, state.selectedSceneId]);

  const validateSelectedMod = useCallback(async () => {
    if (!state.selectedModId) return;
    const modId = state.selectedModId;
    emit({ type: "ModValidationRequested", modId });
    const taskId = `validate-mod:${modId}`;
    dispatch({ type: "taskStarted", task: createTask(taskId, `Validating ${modId}`, "local", "ModInspectorPanel") });
    try {
      const details = await validateMod(modId);
      dispatch({ type: "modDetailsLoaded", details });
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "ModValidationCompleted", modId });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "ModValidationFailed", modId, error: message });
    }
  }, [emit, state.selectedModId]);

  const revealSelectedModFolder = useCallback(async () => {
    if (!state.selectedModId) return;
    const modId = state.selectedModId;
    emit({ type: "RevealPathRequested", pathKind: "mod", modId });
    try {
      const path = await revealModFolder(modId);
      emit({ type: "RevealPathCompleted", pathKind: "mod", path });
    } catch (error) {
      emit({ type: "RevealPathFailed", pathKind: "mod", error: error instanceof Error ? error.message : String(error) });
    }
  }, [emit, state.selectedModId]);

  const revealSelectedSceneDocument = useCallback(async () => {
    if (!state.selectedModId || !state.selectedSceneId) return;
    const modId = state.selectedModId;
    const sceneId = state.selectedSceneId;
    emit({ type: "RevealPathRequested", pathKind: "scene", modId, sceneId });
    try {
      const path = await revealSceneDocument(modId, sceneId);
      emit({ type: "RevealPathCompleted", pathKind: "scene", path });
    } catch (error) {
      emit({ type: "RevealPathFailed", pathKind: "scene", error: error instanceof Error ? error.message : String(error) });
    }
  }, [emit, state.selectedModId, state.selectedSceneId]);

  const revealSelectedProjectFile = useCallback(async () => {
    const modId = state.selectedModId ?? state.modDetails?.id;
    const relativePath = state.selectedFilePath;
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
  }, [emit, state.modDetails?.id, state.selectedFilePath, state.selectedModId]);

  const value = useMemo<EditorStoreValue>(
    () => ({
      state,
      scanMods,
      selectMod,
      loadProjectTree,
      loadEditorSession,
      selectScene,
      selectSceneEntity,
      selectProjectFile,
      selectWorkspaceTab,
      closeWorkspaceTab,
      loadSceneHierarchy,
      regeneratePreview,
      validateSelectedMod,
      revealSelectedModFolder,
      revealSelectedSceneDocument,
      revealSelectedProjectFile,
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
    [closeWorkspaceTab, emit, loadEditorSession, loadProjectTree, loadSceneHierarchy, openSelectedMod, regeneratePreview, revealSelectedModFolder, revealSelectedProjectFile, revealSelectedSceneDocument, scanMods, selectMod, selectProjectFile, selectScene, selectSceneEntity, selectWorkspaceTab, state, validateSelectedMod],
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
