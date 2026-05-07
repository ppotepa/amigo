import { failTask, finishTask } from "../editorTasks";
import { selectedFilePath, selectedModId, selectedSceneId } from "../selectionSelectors";
import type { Action } from "./editorActions";
import type { EditorState } from "./editorState";
import { defaultWorkspaceState, previewKey } from "./editorState";

function workspaceIdFor(actionWorkspaceId?: string): string {
  return actionWorkspaceId ?? "main";
}

function workspaceState(
  state: EditorState,
  actionWorkspaceId?: string,
) {
  const workspaceId = workspaceIdFor(actionWorkspaceId);
  return {
    workspaceId,
    workspace: state.workspaces[workspaceId] ?? defaultWorkspaceState(),
  };
}

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
        workspaces: { main: defaultWorkspaceState() },
        modDetails: null,
      };
    case "selectionChanged": {
      const previousSceneId = selectedSceneId(state.selection);
      const nextSceneId = selectedSceneId(action.selection);
      const nextFilePath = selectedFilePath(action.selection);
      const { workspaceId, workspace } = workspaceState(state);
      const openedFilePaths = nextFilePath
        ? workspace.openedFilePaths.includes(nextFilePath)
          ? workspace.openedFilePaths
          : [...workspace.openedFilePaths, nextFilePath]
        : workspace.openedFilePaths;
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
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId: nextFilePath ? `file:${nextFilePath}` : workspace.activeTabId,
            openedFilePaths,
          },
        },
      };
    }
    case "modDetailsLoaded": {
      const currentModId = selectedModId(state.selection);
      if (currentModId && currentModId !== action.details.id) {
        return state;
      }

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
      {
        const { workspaceId, workspace } = workspaceState(state, action.workspaceId);
        return {
          ...state,
          workspaces: {
            ...state.workspaces,
            [workspaceId]: {
              ...workspace,
              activeTabId: action.tabId,
            },
          },
        };
      }
    case "centerComponentTabOpened": {
      const { workspaceId, workspace } = workspaceState(state, action.workspaceId);
      const centerComponentTabs = workspace.centerComponentTabs.some(
        (instance) => instance.instanceId === action.instance.instanceId,
      )
        ? workspace.centerComponentTabs
        : [...workspace.centerComponentTabs, action.instance];

      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId: action.instance.instanceId,
            centerComponentTabs,
          },
        },
      };
    }
    case "centerComponentTabClosed": {
      const { workspaceId, workspace } = workspaceState(state, action.workspaceId);
      const centerComponentTabs = workspace.centerComponentTabs.filter(
        (instance) => instance.instanceId !== action.instanceId,
      );

      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId: workspace.activeTabId === action.instanceId ? "scene-preview" : workspace.activeTabId,
            centerComponentTabs,
          },
        },
      };
    }
    case "workspaceTabClosed": {
      if (!action.tabId.startsWith("file:")) {
        return state;
      }
      const { workspaceId, workspace } = workspaceState(state, action.workspaceId);
      const relativePath = action.tabId.slice("file:".length);
      const openedFilePaths = workspace.openedFilePaths.filter((path) => path !== relativePath);
      const activeTabId = workspace.activeTabId === action.tabId ? "scene-preview" : workspace.activeTabId;
      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId,
            openedFilePaths,
          },
        },
        selection:
          state.selection.kind === "projectFile" && state.selection.path === relativePath
            ? { kind: "mod", modId: state.selection.modId }
            : state.selection,
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
