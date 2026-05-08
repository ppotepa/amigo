import { useMemo } from "react";
import { Box, FileCode2, Play } from "lucide-react";
import type React from "react";
import type { EditorProjectFileDto, EditorSceneSummaryDto } from "../../api/dto";
import { iconForEditorComponent, editorComponentById } from "../../editor-components/componentRegistry";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import { findProjectFile } from "../../features/files/fileTreeSelectors";
import type { EditorProjectTreeDto } from "../../api/dto";
import { semanticIconClass, toneForComponentDomain, toneForFileKind } from "../../theme/semanticColorRegistry";
import type { WorkspaceTabState } from "../workspaceLayout";

export type WorkspaceTabView = {
  id: string;
  title: string;
  icon: React.ReactNode;
  dirty: boolean;
};

export function useWorkspaceTabs({
  centerComponentTabs,
  dirtyFiles,
  editorModeDirty,
  openedFilePaths,
  projectTree,
  scenePreviewTabId,
  selectedScene,
  workspaceTabs,
}: {
  centerComponentTabs: EditorComponentInstance[];
  dirtyFiles: Record<string, boolean>;
  editorModeDirty: boolean;
  openedFilePaths: string[];
  projectTree?: EditorProjectTreeDto;
  scenePreviewTabId: string;
  selectedScene: EditorSceneSummaryDto | null;
  workspaceTabs?: WorkspaceTabState[];
}): WorkspaceTabView[] {
  return useMemo(() => {
    if (workspaceTabs) {
      return workspaceTabs.filter((tab) => !tab.detachedWorkspaceId).flatMap((tab) => {
        if (tab.id === scenePreviewTabId) {
          return [{
            id: scenePreviewTabId,
            title: selectedScene ? `Scene: ${selectedScene.label}` : "Scene Preview",
            icon: <Play size={13} className="semantic-icon domain-preview" />,
            dirty: editorModeDirty,
          }];
        }

        if (tab.id.startsWith("file:")) {
          const relativePath = tab.id.slice("file:".length);
          const file: EditorProjectFileDto | null = projectTree ? findProjectFile(projectTree.root, relativePath) : null;
          if (!file) return [];

          return [{
            id: `file:${file.relativePath}`,
            title: file.name,
            icon: <FileCode2 size={13} className={semanticIconClass(toneForFileKind(file.kind || file.relativePath))} />,
            dirty: Boolean(dirtyFiles[file.relativePath]),
          }];
        }

        const componentId = tab.componentId ?? tab.pluginId;
        const definition = editorComponentById(componentId);
        return [{
          id: tab.id,
          title: tab.title ?? definition?.title ?? componentId,
          icon: definition ? iconForEditorComponent(definition.icon, 13, toneForComponentDomain(definition.domain)) : <Box size={13} />,
          dirty: tab.dirty,
        }];
      });
    }

    const tabs: WorkspaceTabView[] = selectedScene
      ? [
          {
            id: scenePreviewTabId,
            title: `Scene: ${selectedScene.label}`,
            icon: <Play size={13} className="semantic-icon domain-preview" />,
            dirty: editorModeDirty,
          },
        ]
      : [
          {
            id: scenePreviewTabId,
            title: "Scene Preview",
            icon: <Play size={13} className="semantic-icon domain-preview" />,
            dirty: editorModeDirty,
          },
        ];

    centerComponentTabs.forEach((instance) => {
      const definition = instance.component;
      tabs.push({
        id: instance.instanceId,
        title: instance.titleOverride ?? definition?.title ?? instance.componentId,
        icon: definition ? iconForEditorComponent(definition.icon, 13, toneForComponentDomain(definition.domain)) : <Box size={13} />,
        dirty: false,
      });
    });

    openedFilePaths.forEach((relativePath) => {
      const file: EditorProjectFileDto | null = projectTree ? findProjectFile(projectTree.root, relativePath) : null;
      if (!file) return;

      tabs.push({
        id: `file:${file.relativePath}`,
        title: file.name,
        icon: <FileCode2 size={13} className={semanticIconClass(toneForFileKind(file.kind || file.relativePath))} />,
        dirty: Boolean(dirtyFiles[file.relativePath]),
      });
    });

    return tabs;
  }, [
    centerComponentTabs,
    dirtyFiles,
    editorModeDirty,
    openedFilePaths,
    projectTree,
    scenePreviewTabId,
    selectedScene,
    workspaceTabs,
  ]);
}
