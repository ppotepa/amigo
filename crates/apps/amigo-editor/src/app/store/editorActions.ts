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
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import type { WorkspaceDockProfileId } from "../../main-window/workspaceDockProfiles";
import type { WorkspaceDockLayoutState } from "../../main-window/workspaceLayout";

export type Action =
  | { type: "event"; event: EditorEvent }
  | { type: "windowEvent"; event: WindowBusEvent }
  | { type: "setFileDirty"; path: string; dirty: boolean }
  | { type: "modsLoaded"; mods: EditorModSummaryDto[] }
  | { type: "modSelected"; modId: string }
  | { type: "selectionChanged"; selection: EditorSelectionRef; workspaceId?: string }
  | { type: "workspaceSelectionChanged"; selection: EditorSelectionRef; workspaceId: string }
  | { type: "modDetailsLoaded"; details: EditorModDetailsDto }
  | { type: "projectTreeLoaded"; tree: EditorProjectTreeDto }
  | { type: "projectStructureTreeLoaded"; tree: EditorProjectStructureTreeDto }
  | { type: "projectFileContentLoaded"; content: EditorProjectFileContentDto }
  | { type: "workspaceTabSelected"; tabId: string; workspaceId?: string }
  | { type: "workspaceTabClosed"; tabId: string; workspaceId?: string }
  | { type: "workspaceTabDetached"; sourceWorkspaceId: string; tabId: string; detachedWorkspaceId: string }
  | { type: "workspaceTabAttached"; sourceWorkspaceId: string; targetWorkspaceId: string; tabId: string }
  | { type: "workspaceDockProfileChanged"; workspaceId: string; dockProfileId: WorkspaceDockProfileId }
  | { type: "workspaceDockLayoutChanged"; workspaceId: string; dockLayout: WorkspaceDockLayoutState }
  | { type: "centerComponentTabOpened"; instance: EditorComponentInstance; workspaceId?: string }
  | { type: "centerComponentTabClosed"; instanceId: string; workspaceId?: string }
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
