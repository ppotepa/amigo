import { TargetPanel } from "../../features/target-panel/TargetPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { RIGHT_DOCK, dockable } from "./shared";

export const TargetPanelComponent = dockable({
  id: "target.panel",
  title: "Target",
  category: "workspace",
  debugSource: "src/features/target-panel/TargetPanel.tsx",
  domain: "editor",
  icon: "layout-template",
  description: "Target-driven workbench panel.",
  placement: RIGHT_DOCK,
  defaultPlacement: RIGHT_DOCK,
  allowedPlacements: ["rightDock", "floatingPanel"],
  requiredContext: ["editorSession"],
  render: TargetPanel,
} satisfies Parameters<typeof dockable>[0]);

export const TARGET_PANEL_COMPONENTS: EditorComponentDefinition<any>[] = [
  defineEditorComponent<EditorComponentLaunchContext>()(TargetPanelComponent),
];
