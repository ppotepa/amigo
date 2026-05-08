import type { EditorSelectionRef } from "../app/selectionTypes";
import type { EditorSerializedComponentContext } from "../editor-components/componentTypes";

export type DockAreaId = "left" | "right" | "bottom" | "center";
export type WorkspaceSurfaceMode = "tab" | "detached";
export type WorkspaceDockAreaId = "left" | "rightTop" | "rightBottom" | "bottom" | "center";

export interface DockAreaState {
  visible: boolean;
  size: number;
  tabs: string[];
  activeTabId: string | null;
}

export interface WorkspaceTabState {
  id: string;
  instanceId?: string;
  pluginId: string;
  componentId?: string;
  title: string;
  resourceUri?: string;
  context?: EditorSerializedComponentContext;
  dirty: boolean;
  dockProfileId?: string;
  detachable?: boolean;
  detachedWorkspaceId?: string;
}

export interface WorkspaceDockLayoutState {
  leftDock: DockAreaState;
  rightTopDock: DockAreaState;
  rightBottomDock: DockAreaState;
  bottomDock: DockAreaState;
}

export interface WorkspaceSurfaceState {
  workspaceId: string;
  mode: WorkspaceSurfaceMode;
  sessionId: string | null;
  originTabId?: string;
  title: string;
  tabs: WorkspaceTabState[];
  activeTabId: string;
  dockLayout: WorkspaceDockLayoutState;
  dockProfileId?: string;
  selection: EditorSelectionRef;
}

export interface WorkspaceLayoutState {
  leftDock: DockAreaState;
  rightDock: DockAreaState;
  bottomDock: DockAreaState;
  centerTabs: WorkspaceTabState[];
  activeCenterTabId: string | null;
}

export const DEFAULT_WORKSPACE_LAYOUT: WorkspaceLayoutState = {
  leftDock: {
    visible: true,
    size: 310,
    tabs: ["asset-browser", "files-browser", "scene-browser", "scripts-browser"],
    activeTabId: "asset-browser",
  },
  rightDock: {
    visible: true,
    size: 360,
    tabs: ["inspector", "diagnostics", "properties"],
    activeTabId: "inspector",
  },
  bottomDock: {
    visible: true,
    size: 240,
    tabs: ["problems", "event-log", "tasks", "console", "preview-cache"],
    activeTabId: "problems",
  },
  centerTabs: [
    {
      id: "scene-preview",
      instanceId: "scene.preview:singleton",
      pluginId: "scene-preview",
      componentId: "scene.preview",
      title: "Scene Preview",
      dirty: false,
    },
  ],
  activeCenterTabId: "scene-preview",
};
