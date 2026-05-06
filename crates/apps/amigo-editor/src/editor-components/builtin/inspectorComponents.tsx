import { ChangesPanel } from "../../features/changes/ChangesPanel";
import { InspectorPanel } from "../../features/inspector/InspectorPanel";
import { PropertiesPanel } from "../../features/inspector/PropertiesPanel";
import type { EditorComponentDefinition } from "../componentTypes";
import { RIGHT_DOCK, dockable } from "./shared";

export const INSPECTOR_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "entity.inspector",
    title: "Inspector",
    category: "inspector",
    domain: "scene",
    icon: "box",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: InspectorPanel,
  }),
  dockable({
    id: "entity.properties",
    title: "Properties",
    category: "inspector",
    domain: "scene",
    icon: "box",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: PropertiesPanel,
  }),
  dockable({
    id: "document.changes",
    title: "Changes",
    category: "inspector",
    domain: "scene",
    icon: "list",
    description: "Document changes, undo/redo and save/discard.",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: ChangesPanel,
  }),
];
