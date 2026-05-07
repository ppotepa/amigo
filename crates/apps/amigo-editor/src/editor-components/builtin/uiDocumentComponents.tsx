import { UiDocumentEditor } from "../../editors/ui-document";
import { UiDocumentStructureDock } from "../../editors/ui-document/UiDocumentStructureDock";
import type { EditorComponentDefinition } from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, dockable, workspaceSurface } from "./shared";

export const UI_DOCUMENT_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "ui.document.structure",
    title: "UI Structure",
    category: "ui",
    domain: "ui",
    icon: "list-tree",
    description: "UiDocument tree, node palette and reusable templates.",
    placement: LEFT_DOCK,
    defaultPlacement: LEFT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    render: UiDocumentStructureDock,
  }),
  workspaceSurface({
    id: "ui.document.editor",
    title: "UI Document Editor",
    category: "ui",
    domain: "ui",
    icon: "layout-template",
    description: "Inspect and edit UiDocument node trees.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel"],
    requiredContext: ["editorSession"],
    singleton: false,
    surface: {
      kind: "editor",
      tabMode: true,
      detachedMode: true,
      detachBehavior: "workspace",
      dockProfileId: "ui-document",
    },
    render: UiDocumentEditor,
  }),
];
