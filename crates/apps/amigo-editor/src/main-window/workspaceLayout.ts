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
