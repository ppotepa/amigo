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
  left: string[];
  rightTop: string[];
  rightBottom: string[];
  bottom: string[];
};

export const WORKSPACE_DOCK_PROFILES: Record<WorkspaceDockProfileId, WorkspaceDockProfile> = {
  "ui-document": {
    id: "ui-document",
    left: ["ui.document.structure", "project.explorer", "assets.browser"],
    rightTop: ["entity.properties"],
    rightBottom: ["document.changes", "diagnostics.panel"],
    bottom: ["events.log"],
  },
  "scene-editor": {
    id: "scene-editor",
    left: ["scene.hierarchy", "project.explorer", "assets.browser", "scenes.browser"],
    rightTop: ["scene.context", "entity.properties"],
    rightBottom: ["document.changes", "diagnostics.panel"],
    bottom: ["events.log"],
  },
  "file-viewer": {
    id: "file-viewer",
    left: ["project.explorer", "files.browser"],
    rightTop: ["entity.properties"],
    rightBottom: ["diagnostics.panel"],
    bottom: ["events.log"],
  },
  "asset-editor": {
    id: "asset-editor",
    left: ["project.explorer", "assets.browser"],
    rightTop: ["entity.properties"],
    rightBottom: ["diagnostics.panel", "document.changes"],
    bottom: ["events.log"],
  },
  "project-overview": {
    id: "project-overview",
    left: ["project.explorer"],
    rightTop: ["entity.properties"],
    rightBottom: ["diagnostics.panel"],
    bottom: ["events.log"],
  },
  "default-editor": {
    id: "default-editor",
    left: ["project.explorer", "assets.browser"],
    rightTop: ["entity.properties"],
    rightBottom: ["document.changes", "diagnostics.panel"],
    bottom: ["events.log"],
  },
  minimal: {
    id: "minimal",
    left: ["project.explorer"],
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
