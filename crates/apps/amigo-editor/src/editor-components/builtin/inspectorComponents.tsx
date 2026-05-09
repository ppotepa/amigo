import { ChangesPanel } from "../../features/changes/ChangesPanel";
import { InspectorPanel } from "../../features/inspector/InspectorPanel";
import { ContextPanel } from "../../features/inspector/PropertiesPanel";
import { contextIconForTarget, contextKindLabelForTarget } from "../../editor-targets/editorTargetContextPresentation";
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

export const ContextComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "context.panel",
    title: "Context",
    debugSource: "src/features/inspector/PropertiesPanel.tsx",
    category: "inspector",
    domain: "scene",
    icon: "box",
    placement: RIGHT_DOCK,
    defaultPlacement: RIGHT_DOCK,
    allowedPlacements: ["rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    presentationFromTarget: (target) => ({
      title: `${contextKindLabelForTarget(target)} Context`,
      icon: contextIconForTarget(target),
    }),
    render: ContextPanel,
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
  ContextComponent,
  DocumentChangesComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
