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

export interface EditorState {
  appMode: "startup" | "editor";
  activeSession: OpenModResultDto | null;
  mods: EditorModSummaryDto[];
  selection: EditorSelectionRef;
  activeWorkspaceTabId: string;
  openedFilePaths: string[];
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
  mods: [],
  selection: { kind: "empty" },
  activeWorkspaceTabId: "scene-preview",
  openedFilePaths: [],
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
