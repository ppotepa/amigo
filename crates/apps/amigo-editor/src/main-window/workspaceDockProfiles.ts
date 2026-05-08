import {
  AssetsBrowserComponent,
  DiagnosticsPanelComponent,
  DocumentChangesComponent,
  EntityPropertiesComponent,
  EventsLogComponent,
  FilesBrowserComponent,
  ProjectExplorerComponent,
  SceneContextComponent,
  SceneHierarchyComponent,
  ScenesBrowserComponent,
  TargetContextComponent,
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
  left: readonly EditorComponentDefinition<any>[];
  rightTop: readonly EditorComponentDefinition<any>[];
  rightBottom: readonly EditorComponentDefinition<any>[];
  bottom: readonly EditorComponentDefinition<any>[];
};

export const WORKSPACE_DOCK_PROFILES: Record<WorkspaceDockProfileId, WorkspaceDockProfile> = {
  "ui-document": {
    id: "ui-document",
    left: [UiDocumentStructureComponent, ProjectExplorerComponent, AssetsBrowserComponent],
    rightTop: [EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DocumentChangesComponent, DiagnosticsPanelComponent],
    bottom: [EventsLogComponent],
  },
  "scene-editor": {
    id: "scene-editor",
    left: [SceneHierarchyComponent, ProjectExplorerComponent, AssetsBrowserComponent, ScenesBrowserComponent],
    rightTop: [SceneContextComponent, EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DocumentChangesComponent, DiagnosticsPanelComponent],
    bottom: [EventsLogComponent],
  },
  "file-viewer": {
    id: "file-viewer",
    left: [ProjectExplorerComponent, FilesBrowserComponent],
    rightTop: [EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DiagnosticsPanelComponent],
    bottom: [EventsLogComponent],
  },
  "asset-editor": {
    id: "asset-editor",
    left: [ProjectExplorerComponent, AssetsBrowserComponent],
    rightTop: [EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DiagnosticsPanelComponent, DocumentChangesComponent],
    bottom: [EventsLogComponent],
  },
  "project-overview": {
    id: "project-overview",
    left: [ProjectExplorerComponent],
    rightTop: [EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DiagnosticsPanelComponent],
    bottom: [EventsLogComponent],
  },
  "default-editor": {
    id: "default-editor",
    left: [ProjectExplorerComponent, AssetsBrowserComponent],
    rightTop: [EntityPropertiesComponent],
    rightBottom: [TargetContextComponent, DocumentChangesComponent, DiagnosticsPanelComponent],
    bottom: [EventsLogComponent],
  },
  minimal: {
    id: "minimal",
    left: [ProjectExplorerComponent],
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
