import { failTask, finishTask } from "../editorTasks";
import { selectedFilePath, selectedModId, selectedSceneId } from "../selectionSelectors";
import type { Action } from "./editorActions";
import type { EditorState } from "./editorState";
import { defaultWorkspaceState, previewKey } from "./editorState";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import { serializeComponentContext } from "../../editor-components/componentContextSerialization";
import type { WorkspaceTabState } from "../../main-window/workspaceLayout";
import { normalizeWorkspaceDockProfileId } from "../../main-window/workspaceDockProfiles";

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
    workspace: state.workspaces[workspaceId] ?? defaultWorkspaceState({ workspaceId, selection: state.selection }),
  };
}

function applyWorkspaceSelection(
  state: EditorState,
  actionWorkspaceId: string | undefined,
  nextSelection: EditorState["selection"],
): EditorState {
  const { workspaceId, workspace } = workspaceState(state, actionWorkspaceId);
  const previousSceneId = selectedSceneId(workspace.selection);
  const nextSceneId = selectedSceneId(nextSelection);
  const nextFilePath = selectedFilePath(nextSelection);
  const tabs = nextFilePath ? upsertWorkspaceTab(workspace.tabs, fileWorkspaceTab(nextFilePath)) : workspace.tabs;
  const selection =
    previousSceneId !== nextSceneId && nextSelection.kind === "entity"
      ? {
          kind: "scene" as const,
          modId: nextSelection.modId,
          sceneId: nextSelection.sceneId,
        }
      : nextSelection;

  return {
    ...state,
    activeWorkspaceId: workspaceId,
    selection: workspaceId === "main" ? selection : state.selection,
    workspaces: {
      ...state.workspaces,
      [workspaceId]: {
        ...workspace,
        activeTabId: nextFilePath ? `file:${nextFilePath}` : workspace.activeTabId,
        selection,
        tabs,
      },
    },
  };
}

function upsertWorkspaceTab(tabs: WorkspaceTabState[], tab: WorkspaceTabState): WorkspaceTabState[] {
  return tabs.some((candidate) => candidate.id === tab.id)
    ? tabs.map((candidate) => (candidate.id === tab.id ? { ...candidate, ...tab, detachedWorkspaceId: tab.detachedWorkspaceId } : candidate))
    : [...tabs, tab];
}

function removeWorkspaceTab(tabs: WorkspaceTabState[], tabId: string): WorkspaceTabState[] {
  return tabs.filter((tab) => tab.id !== tabId);
}

function markWorkspaceTabDetached(
  tabs: WorkspaceTabState[],
  tabId: string,
  detachedWorkspaceId: string,
): WorkspaceTabState[] {
  return tabs.map((tab) => (
    tab.id === tabId ? { ...tab, detachedWorkspaceId } : tab
  ));
}

function nextVisibleTabId(tabs: WorkspaceTabState[], currentTabId: string): string {
  return tabs.find((tab) => tab.id !== currentTabId && !tab.detachedWorkspaceId)?.id ?? "scene-preview";
}

function componentWorkspaceTab(instance: EditorComponentInstance): WorkspaceTabState {
  return {
    id: instance.instanceId,
    instanceId: instance.instanceId,
    componentId: instance.componentId,
    title: instance.titleOverride ?? instance.componentId,
    resourceUri: instance.resourceUri,
    context: serializeComponentContext(instance.context),
    dirty: false,
    detachable: true,
  };
}

function fileWorkspaceTab(relativePath: string): WorkspaceTabState {
  return {
    id: `file:${relativePath}`,
    title: relativePath.split("/").pop() ?? relativePath,
    resourceUri: relativePath,
    dirty: false,
    detachable: true,
    dockProfileId: "file-viewer",
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
        activeWorkspaceId: "main",
        workspaces: { main: defaultWorkspaceState({ selection: { kind: "mod", modId: action.modId } }) },
        modDetails: null,
      };
    case "selectionChanged":
      return applyWorkspaceSelection(state, action.workspaceId, action.selection);
    case "workspaceSelectionChanged":
      return applyWorkspaceSelection(state, action.workspaceId, action.selection);
    case "modDetailsLoaded": {
      const currentModId = selectedModId(state.selection);
      if (currentModId && currentModId !== action.details.id) {
        return state;
      }

      const firstScene = action.details.scenes.find((scene) => scene.launcherVisible)?.id ?? action.details.scenes[0]?.id ?? null;
      if (state.selection.kind === "empty" || state.selection.kind === "mod") {
        const selection = firstScene ? { kind: "scene" as const, modId: action.details.id, sceneId: firstScene } : { kind: "mod" as const, modId: action.details.id };
        const mainWorkspace = state.workspaces.main ?? defaultWorkspaceState();
        return {
          ...state,
          modDetails: action.details,
          selection,
          workspaces: {
            ...state.workspaces,
            main: {
              ...mainWorkspace,
              selection,
            },
          },
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
          activeWorkspaceId: workspaceId,
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
      const tabs = upsertWorkspaceTab(workspace.tabs, componentWorkspaceTab(action.instance));

      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId: action.instance.instanceId,
            tabs,
          },
        },
      };
    }
    case "centerComponentTabClosed": {
      const { workspaceId, workspace } = workspaceState(state, action.workspaceId);
      const tabs = removeWorkspaceTab(workspace.tabs, action.instanceId);

      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId: workspace.activeTabId === action.instanceId ? "scene-preview" : workspace.activeTabId,
            tabs,
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
      const activeTabId = workspace.activeTabId === action.tabId ? "scene-preview" : workspace.activeTabId;
      const tabs = removeWorkspaceTab(workspace.tabs, action.tabId);
      const selection =
        workspace.selection.kind === "projectFile" && workspace.selection.path === relativePath
          ? { kind: "mod" as const, modId: workspace.selection.modId }
          : workspace.selection;
      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [workspaceId]: {
            ...workspace,
            activeTabId,
            selection,
            tabs,
          },
        },
        selection:
          workspaceId === "main" && state.selection.kind === "projectFile" && state.selection.path === relativePath
            ? { kind: "mod", modId: state.selection.modId }
            : state.selection,
      };
    }
    case "workspaceTabDetached": {
      const { workspace } = workspaceState(state, action.sourceWorkspaceId);
      const tabs = markWorkspaceTabDetached(workspace.tabs, action.tabId, action.detachedWorkspaceId);
      const activeTabId = workspace.activeTabId === action.tabId ? nextVisibleTabId(tabs, action.tabId) : workspace.activeTabId;
      const detachedSourceTab = workspace.tabs.find((tab) => tab.id === action.tabId);
      if (!detachedSourceTab) {
        return state;
      }
      const detachedTab = {
        ...detachedSourceTab,
        detachedWorkspaceId: undefined,
      };
      const detachedWorkspace = defaultWorkspaceState({
        workspaceId: action.detachedWorkspaceId,
        mode: "detached",
        sessionId: workspace.sessionId,
        originTabId: action.tabId,
        title: detachedTab.title,
        dockProfileId: normalizeWorkspaceDockProfileId(detachedTab.dockProfileId ?? workspace.dockProfileId),
        dockLayout: workspace.dockLayout,
        activeTabId: detachedTab.id,
        tabs: [detachedTab],
        selection: workspace.selection,
      });
      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [action.sourceWorkspaceId]: {
            ...workspace,
            activeTabId,
            tabs,
          },
          [action.detachedWorkspaceId]: detachedWorkspace,
        },
      };
    }
    case "workspaceTabAttached": {
      const { workspace: sourceWorkspace } = workspaceState(state, action.sourceWorkspaceId);
      const { workspace: targetWorkspace } = workspaceState(state, action.targetWorkspaceId);
      const sourceTab = sourceWorkspace.tabs.find((tab) => tab.id === action.tabId);
      if (!sourceTab) {
        return state;
      }

      const attachedTab = { ...sourceTab, detachedWorkspaceId: undefined };
      const targetTabs = upsertWorkspaceTab(
        targetWorkspace.tabs.map((tab) => (
          tab.id === action.tabId ? { ...tab, detachedWorkspaceId: undefined } : tab
        )),
        attachedTab,
      );
      const sourceTabs = removeWorkspaceTab(sourceWorkspace.tabs, action.tabId);
      const workspaces = {
        ...state.workspaces,
        [action.targetWorkspaceId]: {
          ...targetWorkspace,
          activeTabId: action.tabId,
          tabs: targetTabs,
          selection: sourceWorkspace.selection,
        },
        [action.sourceWorkspaceId]: {
          ...sourceWorkspace,
          tabs: sourceTabs,
          activeTabId: sourceWorkspace.activeTabId === action.tabId ? nextVisibleTabId(sourceTabs, action.tabId) : sourceWorkspace.activeTabId,
        },
      };

      if (sourceWorkspace.mode === "detached" && sourceTabs.length === 0) {
        delete workspaces[action.sourceWorkspaceId];
      }

      return {
        ...state,
        activeWorkspaceId: action.targetWorkspaceId,
        selection: action.targetWorkspaceId === "main" ? sourceWorkspace.selection : state.selection,
        workspaces,
      };
    }
    case "workspaceDockProfileChanged": {
      const { workspace } = workspaceState(state, action.workspaceId);
      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [action.workspaceId]: {
            ...workspace,
            dockProfileId: action.dockProfileId,
          },
        },
      };
    }
    case "workspaceDockLayoutChanged": {
      const { workspace } = workspaceState(state, action.workspaceId);
      return {
        ...state,
        workspaces: {
          ...state.workspaces,
          [action.workspaceId]: {
            ...workspace,
            dockLayout: action.dockLayout,
          },
        },
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
