import { SceneHierarchyPanel } from "../../features/scenes/SceneHierarchyPanel";
import { ScenePreviewWorkbench } from "../../features/scenes/ScenePreviewWorkbench";
import { ScenesBrowserPanel } from "../../features/scenes/ScenesBrowserPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type {
  EditorComponentDefinition,
  EditorComponentLaunchContext,
  ScenePreviewComponentContext,
} from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, RIGHT_DOCK, dockable, workspaceSurface } from "./shared";

export const ScenesBrowserComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "scenes.browser",
    title: "Scenes",
    debugSource: "src/features/scenes/ScenesBrowserPanel.tsx",
    category: "explorer",
    domain: "scene",
    icon: "play",
    description: "Scene navigation list.",
    placement: LEFT_DOCK,
    defaultPlacement: LEFT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    toolbar: {
      compact: true,
      controls: [
        {
          kind: "segmented",
          id: "viewMode",
          label: "View",
          defaultValue: "list",
          options: [
            { id: "list", label: "List", icon: "list" },
            { id: "status", label: "Status", icon: "alert-triangle" },
          ],
        },
        { kind: "toggle", id: "visibleOnly", label: "Visible", icon: "play", defaultValue: false },
      ],
    },
    render: ScenesBrowserPanel,
  }),
);

export const SceneHierarchyComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "scene.hierarchy",
    title: "Scene Hierarchy",
    debugSource: "src/features/scenes/SceneHierarchyPanel.tsx",
    category: "explorer",
    domain: "scene",
    icon: "list-tree",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["selectedScene"],
    render: SceneHierarchyPanel,
  }),
);

export const ScenePreviewComponent = defineEditorComponent<ScenePreviewComponentContext>()(
  workspaceSurface({
    id: "scene.preview",
    title: "Scene Preview",
    debugSource: "src/features/scenes/ScenePreviewWorkbench.tsx",
    category: "preview",
    domain: "preview",
    subdomain: "scene",
    icon: "play",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
    requiredContext: ["selectedScene"],
    canOpenInCenterTabs: false,
    surface: {
      kind: "editor",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "scene-editor",
    },
    render: ScenePreviewWorkbench,
  }),
);

export const SCENE_COMPONENTS = [
  ScenesBrowserComponent,
  SceneHierarchyComponent,
  ScenePreviewComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
