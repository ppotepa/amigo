import type {
  EditorModDetailsDto,
  EditorModSummaryDto,
  EditorProjectFileContentDto,
  EditorProjectStructureTreeDto,
  EditorProjectTreeDto,
  EditorSceneHierarchyDto,
  OpenModResultDto,
  ScenePreviewDto,
} from "../../api/dto";
import type { EditorEvent } from "../editorEvents";
import type { EditorTask } from "../editorTasks";
import type { EditorSelectionRef } from "../selectionTypes";
import type { WindowBusEvent } from "../windowBusTypes";
import type {
  WorkspaceDockLayoutState,
  WorkspaceSurfaceMode,
  WorkspaceTabState,
} from "../../main-window/workspaceLayout";
import { singletonComponentInstanceId } from "../../editor-components/componentInstances";
import { ScenePreviewComponent } from "../../editor-components/componentRegistry";
import {
  WORKSPACE_DOCK_PROFILES,
  type WorkspaceDockProfileId,
} from "../../main-window/workspaceDockProfiles";

export interface WorkspaceState {
  workspaceId: string;
  mode: WorkspaceSurfaceMode;
  sessionId: string | null;
  originTabId?: string;
  title: string;
  dockProfileId: WorkspaceDockProfileId;
  dockLayout: WorkspaceDockLayoutState;
  activeTabId: string;
  tabs: WorkspaceTabState[];
  selection: EditorSelectionRef;
}

export interface EditorState {
  appMode: "startup" | "editor";
  activeSession: OpenModResultDto | null;
  activeWorkspaceId: string;
  mods: EditorModSummaryDto[];
  selection: EditorSelectionRef;
  workspaces: Record<string, WorkspaceState>;
  modDetails: EditorModDetailsDto | null;
  projectTrees: Record<string, EditorProjectTreeDto>;
  projectStructureTrees: Record<string, EditorProjectStructureTreeDto>;
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

export const initialState: EditorState = {
  appMode: "startup",
  activeSession: null,
  activeWorkspaceId: "main",
  mods: [],
  selection: { kind: "empty" },
  workspaces: {
    main: {
      workspaceId: "main",
      mode: "tab",
      sessionId: null,
      title: "Main Workspace",
      dockProfileId: "scene-editor",
      dockLayout: defaultWorkspaceDockLayout(),
      activeTabId: "scene-preview",
      tabs: defaultWorkspaceTabs(),
      selection: { kind: "empty" },
    },
  },
  modDetails: null,
  projectTrees: {},
  projectStructureTrees: {},
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

export function previewKey(modId: string, sceneId: string): string {
  return `${modId}:${sceneId}`;
}

export function defaultWorkspaceState(overrides: Partial<WorkspaceState> = {}): WorkspaceState {
  const workspaceId = overrides.workspaceId ?? "main";
  return {
    workspaceId,
    mode: overrides.mode ?? "tab",
    sessionId: overrides.sessionId ?? null,
    originTabId: overrides.originTabId,
    title: overrides.title ?? (workspaceId === "main" ? "Main Workspace" : "Workspace"),
    dockProfileId: overrides.dockProfileId ?? "scene-editor",
    dockLayout: overrides.dockLayout ?? defaultWorkspaceDockLayout(),
    activeTabId: "scene-preview",
    tabs: defaultWorkspaceTabs(),
    selection: { kind: "empty" },
    ...overrides,
  };
}

export function defaultWorkspaceTabs(): WorkspaceTabState[] {
  return [
    {
      id: "scene-preview",
      instanceId: singletonComponentInstanceId(ScenePreviewComponent),
      componentId: ScenePreviewComponent.id,
      title: "Scene Preview",
      dirty: false,
      detachable: true,
      dockProfileId: "scene-editor",
    },
  ];
}

export function defaultWorkspaceDockLayout(): WorkspaceDockLayoutState {
  const profile = WORKSPACE_DOCK_PROFILES["scene-editor"];
  const slotId = (slot: (typeof profile.left)[number]): string => slot.component.id;
  return {
    leftDock: {
      visible: true,
      size: 360,
      tabs: profile.left.map(slotId),
      activeTabId: profile.left[0] ? slotId(profile.left[0]) : null,
    },
    rightTopDock: {
      visible: true,
      size: 380,
      tabs: profile.rightTop.map(slotId),
      activeTabId: profile.rightTop[0] ? slotId(profile.rightTop[0]) : null,
    },
    rightBottomDock: {
      visible: true,
      size: 280,
      tabs: profile.rightBottom.map(slotId),
      activeTabId: profile.rightBottom[0] ? slotId(profile.rightBottom[0]) : null,
    },
    bottomDock: {
      visible: true,
      size: 260,
      tabs: profile.bottom.map(slotId),
      activeTabId: profile.bottom[0] ? slotId(profile.bottom[0]) : null,
    },
  };
}
