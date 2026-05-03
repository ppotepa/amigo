export type DockAreaId = "left" | "right" | "bottom" | "center";

export interface DockAreaState {
  visible: boolean;
  size: number;
  tabs: string[];
  activeTabId: string | null;
}

export interface WorkspaceTabState {
  id: string;
  pluginId: string;
  title: string;
  resourceUri?: string;
  dirty: boolean;
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
    tabs: ["project-explorer", "asset-browser", "scene-hierarchy"],
    activeTabId: "project-explorer",
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
      pluginId: "scene-preview",
      title: "Scene Preview",
      dirty: false,
    },
  ],
  activeCenterTabId: "scene-preview",
};
