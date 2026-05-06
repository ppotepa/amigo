import { SceneContextDock } from "../../features/scenes/context/SceneContextDock";
import { SceneHierarchyPanel } from "../../features/scenes/SceneHierarchyPanel";
import { ScenePreviewWorkbench } from "../../features/scenes/ScenePreviewWorkbench";
import { ScenesBrowserPanel } from "../../features/scenes/ScenesBrowserPanel";
import type { EditorComponentDefinition } from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, RIGHT_DOCK, centerTab, dockable } from "./shared";

export const SCENE_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "scenes.browser",
    title: "Scenes",
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
  dockable({
    id: "scene.context",
    title: "Scene Context",
    category: "inspector",
    domain: "scene",
    icon: "list-tree",
    description: "Context widgets for the active scene.",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["selectedScene"],
    render: SceneContextDock,
  }),
  dockable({
    id: "scene.hierarchy",
    title: "Scene Hierarchy",
    category: "explorer",
    domain: "scene",
    icon: "list-tree",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["selectedScene"],
    render: SceneHierarchyPanel,
  }),
  centerTab({
    id: "scene.preview",
    title: "Scene Preview",
    category: "preview",
    domain: "preview",
    subdomain: "scene",
    icon: "play",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
    requiredContext: ["selectedScene"],
    canOpenInCenterTabs: false,
    render: ScenePreviewWorkbench,
  }),
];
