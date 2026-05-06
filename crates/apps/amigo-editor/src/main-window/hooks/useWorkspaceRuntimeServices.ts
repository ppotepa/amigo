import { useMemo } from "react";
import type { WorkspaceRuntimeServices } from "../workspaceRuntimeServices";

export function useWorkspaceRuntimeServices(input: WorkspaceRuntimeServices): WorkspaceRuntimeServices {
  return useMemo(
    () => input,
    [
      input.allProblems,
      input.assetRegistry,
      input.details,
      input.editorFrame,
      input.editorModeSession,
      input.editorPreviewSync,
      input.editorSnapshot,
      input.eventFilter,
      input.eventRows,
      input.eventSearch,
      input.eventSessionFilter,
      input.eventSourceFilter,
      input.hierarchy,
      input.hierarchyTask,
      input.preview,
      input.previewPlaying,
      input.previewTask,
      input.projectStructureTree,
      input.projectTree,
      input.projectTreeTask,
      input.selection,
      input.selectedAsset,
      input.selectedEntity,
      input.selectedFile,
      input.selectedFileContent,
      input.selectedScene,
      input.selectedUiNode,
      input.selectedUiNodeObject,
      input.tasks,
      input.toolbarState,
      input.windowEventRows,
    ],
  );
}
