import type { EditorProjectFileDto, EditorProjectTreeDto } from "../api/dto";
import { createComponentInstance } from "../editor-components/componentInstances";
import { resolveSerializedComponentRef } from "../editor-components/componentRefSerialization";
import type { EditorComponentInstance } from "../editor-components/componentTypes";
import { findProjectFile } from "../features/files/fileTreeSelectors";
import type { WorkspaceTabState } from "./workspaceLayout";

export function centerComponentInstancesFromTabs({
  sessionId,
  tabs,
}: {
  sessionId?: string | null;
  tabs: WorkspaceTabState[];
}): EditorComponentInstance[] {
  return tabs
    .filter((tab) => !tab.detachedWorkspaceId && tab.componentId && tab.id !== "scene-preview")
    .flatMap((tab) => {
      const resolved = resolveSerializedComponentRef({
        componentId: tab.componentId ?? "",
        context: tab.context,
      });
      if (!resolved) return [];

      return [{
      ...createComponentInstance({
        component: resolved.component,
        context: resolved.context,
        placement: { kind: "centerTab" },
        resourceUri: tab.resourceUri,
        sessionId: sessionId ?? undefined,
        titleOverride: tab.title,
      }),
      instanceId: tab.id,
    }];
    });
}

export function openedFilePathsFromTabs(tabs: WorkspaceTabState[]): string[] {
  return tabs
    .filter((tab) => !tab.detachedWorkspaceId && tab.id.startsWith("file:"))
    .map((tab) => tab.id.slice("file:".length));
}

export function activeFileFromWorkspaceTab({
  activeTabId,
  projectTree,
  selectedFile,
}: {
  activeTabId: string;
  projectTree?: EditorProjectTreeDto;
  selectedFile: EditorProjectFileDto | null;
}): EditorProjectFileDto | null {
  if (!activeTabId.startsWith("file:")) {
    return selectedFile;
  }

  const relativePath = activeTabId.slice("file:".length);
  return (
    (projectTree ? findProjectFile(projectTree.root, relativePath) : null) ??
    (selectedFile?.relativePath === relativePath ? selectedFile : null)
  );
}
