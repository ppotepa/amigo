import { ChangesPanel } from "../../features/changes/ChangesPanel";
import { InspectorPanel } from "../../features/inspector/InspectorPanel";
import { PropertiesPanel } from "../../features/inspector/PropertiesPanel";
import { TargetContextPanel } from "../../features/target-context/TargetContextPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { RIGHT_DOCK, dockable } from "./shared";

export const EntityInspectorComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "entity.inspector",
    title: "Inspector",
    debugSource: "src/features/inspector/InspectorPanel.tsx",
    category: "inspector",
    domain: "scene",
    icon: "box",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: InspectorPanel,
  }),
);

export const EntityPropertiesComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "entity.properties",
    title: "Item Context",
    debugSource: "src/features/inspector/PropertiesPanel.tsx",
    category: "inspector",
    domain: "scene",
    icon: "box",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: PropertiesPanel,
  }),
);

export const TargetContextComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "target.context",
    title: "Target Context",
    debugSource: "src/features/target-context/TargetContextPanel.tsx",
    category: "inspector",
    domain: "editor",
    icon: "gauge",
    description: "Context panels for the active editor target.",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: TargetContextPanel,
  }),
);

export const DocumentChangesComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "document.changes",
    title: "Changes",
    debugSource: "src/features/changes/ChangesPanel.tsx",
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
);

export const INSPECTOR_COMPONENTS = [
  EntityInspectorComponent,
  EntityPropertiesComponent,
  TargetContextComponent,
  DocumentChangesComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
