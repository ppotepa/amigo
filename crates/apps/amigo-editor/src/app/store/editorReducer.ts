import { failTask, finishTask } from "../editorTasks";
import { selectedFilePath, selectedSceneId } from "../selectionSelectors";
import type { Action } from "./editorActions";
import type { EditorState } from "./editorState";
import { previewKey } from "./editorState";

export function reducer(state: EditorState, action: Action): EditorState {
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
        selection: { kind: "mod", modId: action.modId },
        activeWorkspaceTabId: "scene-preview",
        openedFilePaths: [],
        modDetails: null,
      };
    case "selectionChanged": {
      const previousSceneId = selectedSceneId(state.selection);
      const nextSceneId = selectedSceneId(action.selection);
      const nextFilePath = selectedFilePath(action.selection);
      const openedFilePaths = nextFilePath
        ? state.openedFilePaths.includes(nextFilePath)
          ? state.openedFilePaths
          : [...state.openedFilePaths, nextFilePath]
        : state.openedFilePaths;
      const selection =
        previousSceneId !== nextSceneId && action.selection.kind === "entity"
          ? {
              kind: "scene" as const,
              modId: action.selection.modId,
              sceneId: action.selection.sceneId,
            }
          : action.selection;

      return {
        ...state,
        selection,
        activeWorkspaceTabId: nextFilePath ? `file:${nextFilePath}` : "scene-preview",
        openedFilePaths,
      };
    }
    case "modDetailsLoaded": {
      const firstScene = action.details.scenes.find((scene) => scene.launcherVisible)?.id ?? action.details.scenes[0]?.id ?? null;
      if (state.selection.kind === "empty" || state.selection.kind === "mod") {
        return {
          ...state,
          modDetails: action.details,
          selection: firstScene ? { kind: "scene", modId: action.details.id, sceneId: firstScene } : { kind: "mod", modId: action.details.id },
        };
      }
      return { ...state, modDetails: action.details };
    }
    case "projectTreeLoaded":
      return { ...state, projectTrees: { ...state.projectTrees, [action.tree.modId]: action.tree } };
    case "projectStructureTreeLoaded":
      return { ...state, projectStructureTrees: { ...state.projectStructureTrees, [action.tree.modId]: action.tree } };
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
        selection:
          state.selection.kind === "projectFile" && state.selection.path === relativePath
            ? { kind: "mod", modId: state.selection.modId }
            : state.selection,
        activeWorkspaceTabId,
      };
    }
    case "previewLoaded":
      return { ...state, previews: { ...state.previews, [previewKey(action.preview.modId, action.preview.sceneId)]: action.preview } };
    case "sceneHierarchyLoaded":
      if (state.selection.kind === "entity" && state.selection.sceneId === action.hierarchy.sceneId) {
        const currentSelection = state.selection;
        const entityId =
          action.hierarchy.entities.find((entity) => entity.id === currentSelection.entityId)?.id ??
          action.hierarchy.entities[0]?.id ??
          currentSelection.entityId;
        return {
          ...state,
          selection: {
            kind: "entity",
            modId: currentSelection.modId,
            sceneId: currentSelection.sceneId,
            entityId,
          },
          sceneHierarchies: {
            ...state.sceneHierarchies,
            [previewKey(action.hierarchy.modId, action.hierarchy.sceneId)]: action.hierarchy,
          },
        };
      }
      return {
        ...state,
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
