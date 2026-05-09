import {
  AssetsBrowserComponent,
  CachePreviewComponent,
  TargetPanelComponent,
  DiagnosticsProblemsComponent,
  DiagnosticsPanelComponent,
  DocumentChangesComponent,
  EventsLogComponent,
  FilesBrowserComponent,
  ProjectExplorerComponent,
  SceneHierarchyComponent,
  ScenesBrowserComponent,
  ScriptsBrowserComponent,
  ScriptingConsoleComponent,
  TasksMonitorComponent,
  UiDocumentStructureComponent,
} from "../editor-components/componentRegistry";
import type { EditorComponentDefinition } from "../editor-components/componentTypes";

export type WorkspaceDockProfileId =
  | "scene-editor"
  | "ui-document"
  | "file-viewer"
  | "asset-editor"
  | "project-overview"
  | "default-editor"
  | "minimal";

export type WorkspaceDockProfile = {
  id: WorkspaceDockProfileId;
  left: readonly WorkspaceDockSlot[];
  rightTop: readonly WorkspaceDockSlot[];
  rightBottom: readonly WorkspaceDockSlot[];
  bottom: readonly WorkspaceDockSlot[];
};

export type WorkspaceDockSlot = { kind: "component"; component: EditorComponentDefinition<any> };

export function componentSlot(component: EditorComponentDefinition<any>): WorkspaceDockSlot {
  return { kind: "component", component };
}

const STANDARD_BOTTOM_DOCK = [
  componentSlot(DiagnosticsProblemsComponent),
  componentSlot(EventsLogComponent),
  componentSlot(TasksMonitorComponent),
  componentSlot(ScriptingConsoleComponent),
  componentSlot(CachePreviewComponent),
] as const;

const PROJECT_BOTTOM_DOCK = [
  componentSlot(FilesBrowserComponent),
  componentSlot(ScriptsBrowserComponent),
  ...STANDARD_BOTTOM_DOCK,
] as const;

export const WORKSPACE_DOCK_PROFILES: Record<WorkspaceDockProfileId, WorkspaceDockProfile> = {
  "ui-document": {
    id: "ui-document",
    left: [componentSlot(UiDocumentStructureComponent), componentSlot(ProjectExplorerComponent), componentSlot(AssetsBrowserComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DocumentChangesComponent), componentSlot(DiagnosticsPanelComponent)],
    bottom: STANDARD_BOTTOM_DOCK,
  },
  "scene-editor": {
    id: "scene-editor",
    left: [componentSlot(SceneHierarchyComponent), componentSlot(ProjectExplorerComponent), componentSlot(AssetsBrowserComponent), componentSlot(ScenesBrowserComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DocumentChangesComponent), componentSlot(DiagnosticsPanelComponent)],
    bottom: PROJECT_BOTTOM_DOCK,
  },
  "file-viewer": {
    id: "file-viewer",
    left: [componentSlot(ProjectExplorerComponent), componentSlot(FilesBrowserComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DiagnosticsPanelComponent)],
    bottom: PROJECT_BOTTOM_DOCK,
  },
  "asset-editor": {
    id: "asset-editor",
    left: [componentSlot(ProjectExplorerComponent), componentSlot(AssetsBrowserComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DiagnosticsPanelComponent), componentSlot(DocumentChangesComponent)],
    bottom: PROJECT_BOTTOM_DOCK,
  },
  "project-overview": {
    id: "project-overview",
    left: [componentSlot(ProjectExplorerComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DiagnosticsPanelComponent)],
    bottom: STANDARD_BOTTOM_DOCK,
  },
  "default-editor": {
    id: "default-editor",
    left: [componentSlot(ProjectExplorerComponent), componentSlot(AssetsBrowserComponent)],
    rightTop: [componentSlot(TargetPanelComponent)],
    rightBottom: [componentSlot(DocumentChangesComponent), componentSlot(DiagnosticsPanelComponent)],
    bottom: PROJECT_BOTTOM_DOCK,
  },
  minimal: {
    id: "minimal",
    left: [componentSlot(ProjectExplorerComponent)],
    rightTop: [],
    rightBottom: [],
    bottom: [],
  },
};

export function workspaceDockProfileForComponent(
  component: EditorComponentDefinition | null | undefined,
): WorkspaceDockProfile {
  const profileId = normalizeWorkspaceDockProfileId(component?.surface?.dockProfileId);
  return WORKSPACE_DOCK_PROFILES[profileId];
}

export function normalizeWorkspaceDockProfileId(
  profileId: string | null | undefined,
): WorkspaceDockProfileId {
  if (profileId && profileId in WORKSPACE_DOCK_PROFILES) {
    return profileId as WorkspaceDockProfileId;
  }

  return "default-editor";
}
