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

export type Action =
  | { type: "event"; event: EditorEvent }
  | { type: "windowEvent"; event: WindowBusEvent }
  | { type: "setFileDirty"; path: string; dirty: boolean }
  | { type: "modsLoaded"; mods: EditorModSummaryDto[] }
  | { type: "modSelected"; modId: string }
  | { type: "selectionChanged"; selection: EditorSelectionRef }
  | { type: "modDetailsLoaded"; details: EditorModDetailsDto }
  | { type: "projectTreeLoaded"; tree: EditorProjectTreeDto }
  | { type: "projectStructureTreeLoaded"; tree: EditorProjectStructureTreeDto }
  | { type: "projectFileContentLoaded"; content: EditorProjectFileContentDto }
  | { type: "workspaceTabSelected"; tabId: string }
  | { type: "workspaceTabClosed"; tabId: string }
  | { type: "previewLoaded"; preview: ScenePreviewDto }
  | { type: "sceneHierarchyLoaded"; hierarchy: EditorSceneHierarchyDto }
  | { type: "taskStarted"; task: EditorTask }
  | { type: "taskFinished"; taskId: string }
  | { type: "taskFailed"; taskId: string; error: string }
  | { type: "taskProgress"; taskId: string; progress: number }
  | { type: "sessionOpened"; session: OpenModResultDto }
  | { type: "returnToStartup" }
  | { type: "toggleInspectorSection"; sectionId: string }
  | { type: "setPreviewPlaying"; playing: boolean }
  | { type: "setContentFilter"; filter: string | null };
