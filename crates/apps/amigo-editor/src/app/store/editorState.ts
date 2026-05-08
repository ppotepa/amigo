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
import type { WorkspaceDockProfileId } from "../../main-window/workspaceDockProfiles";

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
      instanceId: "scene.preview:singleton",
      componentId: "scene.preview",
      title: "Scene Preview",
      dirty: false,
      detachable: true,
      dockProfileId: "scene-editor",
    },
  ];
}

export function defaultWorkspaceDockLayout(): WorkspaceDockLayoutState {
  return {
    leftDock: {
      visible: true,
      size: 360,
      tabs: ["scene.hierarchy", "project.explorer", "assets.browser", "scenes.browser"],
      activeTabId: "scene.hierarchy",
    },
    rightTopDock: {
      visible: true,
      size: 380,
      tabs: ["scene.context", "entity.properties"],
      activeTabId: "scene.context",
    },
    rightBottomDock: {
      visible: true,
      size: 280,
      tabs: ["document.changes", "diagnostics.panel"],
      activeTabId: "document.changes",
    },
    bottomDock: {
      visible: true,
      size: 260,
      tabs: ["events.log"],
      activeTabId: "events.log",
    },
  };
}
