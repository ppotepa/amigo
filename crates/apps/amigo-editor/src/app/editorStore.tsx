import React, { createContext, useCallback, useContext, useEffect, useMemo, useReducer } from "react";
import { listen } from "@tauri-apps/api/event";
import { closeEditorSession, getModDetails, listKnownMods, openMod, requestScenePreview, revealModFolder, revealSceneDocument, validateMod } from "../api/editorApi";
import type { EditorModDetailsDto, EditorModSummaryDto, EditorSceneSummaryDto, OpenModResultDto, ScenePreviewDto } from "../api/dto";
import type { EditorEvent } from "./editorEvents";
import type { EditorTask } from "./editorTasks";
import { createTask, failTask, finishTask } from "./editorTasks";

interface EditorState {
  appMode: "startup" | "editor";
  activeSession: OpenModResultDto | null;
  mods: EditorModSummaryDto[];
  selectedModId: string | null;
  selectedSceneId: string | null;
  modDetails: EditorModDetailsDto | null;
  previews: Record<string, ScenePreviewDto>;
  tasks: Record<string, EditorTask>;
  events: EditorEvent[];
  previewPlaying: boolean;
  contentFilter: string | null;
  openInspectorSections: Record<string, boolean>;
}

const initialState: EditorState = {
  appMode: "startup",
  activeSession: null,
  mods: [],
  selectedModId: null,
  selectedSceneId: null,
  modDetails: null,
  previews: {},
  tasks: {},
  events: [],
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
};

type Action =
  | { type: "event"; event: EditorEvent }
  | { type: "modsLoaded"; mods: EditorModSummaryDto[] }
  | { type: "modSelected"; modId: string }
  | { type: "modDetailsLoaded"; details: EditorModDetailsDto }
  | { type: "sceneSelected"; sceneId: string }
  | { type: "previewLoaded"; preview: ScenePreviewDto }
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

function reducer(state: EditorState, action: Action): EditorState {
  switch (action.type) {
    case "event":
      return { ...state, events: [action.event, ...state.events].slice(0, 80) };
    case "modsLoaded":
      return { ...state, mods: action.mods };
    case "modSelected":
      return { ...state, selectedModId: action.modId, selectedSceneId: null, modDetails: null };
    case "modDetailsLoaded": {
      const firstScene = action.details.scenes.find((scene) => scene.launcherVisible)?.id ?? action.details.scenes[0]?.id ?? null;
      return { ...state, modDetails: action.details, selectedSceneId: state.selectedSceneId ?? firstScene };
    }
    case "sceneSelected":
      return { ...state, selectedSceneId: action.sceneId };
    case "previewLoaded":
      return { ...state, previews: { ...state.previews, [previewKey(action.preview.modId, action.preview.sceneId)]: action.preview } };
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
  selectScene: (scene: EditorSceneSummaryDto) => Promise<void>;
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

    void listen<{ modId: string; sceneId: string; current: number; total: number }>(
      "scene-preview-frame-generated",
      (event) => {
        if (cancelled) {
          return;
        }
        const payload = event.payload;
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

  const selectScene = useCallback(
    async (scene: EditorSceneSummaryDto) => {
      if (!state.selectedModId) return;
      const modId = state.selectedModId;
      dispatch({ type: "sceneSelected", sceneId: scene.id });
      emit({ type: "SceneSelected", modId, sceneId: scene.id });

      if (!state.previews[previewKey(modId, scene.id)]) {
          await regeneratePreview(modId, scene.id, false);
      }
    },
    [emit, regeneratePreview, state.previews, state.selectedModId],
  );

  const loadModDetails = useCallback(
    async (modId: string) => {
      emit({ type: "ModDetailsRequested", modId });
      const taskId = `load-mod-details:${modId}`;
      dispatch({ type: "taskStarted", task: createTask(taskId, `Loading ${modId}`, "local", "ModInspectorPanel") });

      try {
        const details = await getModDetails(modId);
        dispatch({ type: "modDetailsLoaded", details });
        dispatch({ type: "taskFinished", taskId });
        emit({ type: "ModDetailsLoaded", modId: details.id });
        const firstScene = details.scenes.find((scene) => scene.launcherVisible) ?? details.scenes[0];
        if (firstScene) {
          dispatch({ type: "sceneSelected", sceneId: firstScene.id });
          emit({ type: "SceneSelected", modId: details.id, sceneId: firstScene.id });
          await regeneratePreview(details.id, firstScene.id, false);
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        dispatch({ type: "taskFailed", taskId, error: message });
        emit({ type: "ModDetailsFailed", modId, error: message });
      }
    },
    [emit, regeneratePreview],
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
      const result = await openMod(modId);
      dispatch({ type: "sessionOpened", session: result });
      dispatch({ type: "taskFinished", taskId });
      emit({ type: "OpenModCompleted", modId: result.modId, sessionId: result.sessionId });
      emit({ type: "MainEditorWindowRequested", modId: result.modId, sessionId: result.sessionId });
      emit({ type: "EditorSessionLoaded", modId: result.modId, sessionId: result.sessionId });
      emit({ type: "DockLayoutLoaded", layoutId: "default" });
      emit({ type: "WorkspaceReady", sessionId: result.sessionId });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "taskFailed", taskId, error: message });
      emit({ type: "OpenModFailed", modId, error: message });
    }
  }, [emit, state.selectedModId]);

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

  const value = useMemo<EditorStoreValue>(
    () => ({
      state,
      scanMods,
      selectMod,
      selectScene,
      regeneratePreview,
      validateSelectedMod,
      revealSelectedModFolder,
      revealSelectedSceneDocument,
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
    }),
    [emit, openSelectedMod, regeneratePreview, revealSelectedModFolder, revealSelectedSceneDocument, scanMods, selectMod, selectScene, state, validateSelectedMod],
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
