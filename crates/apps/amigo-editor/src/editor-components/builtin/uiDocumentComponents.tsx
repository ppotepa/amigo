import { UiDocumentEditor } from "../../editors/ui-document";
import { UiDocumentStructureDock } from "../../editors/ui-document/UiDocumentStructureDock";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type {
  EditorComponentDefinition,
  EditorComponentLaunchContext,
  UiDocumentEditorContext,
} from "../componentTypes";
import { CENTER_TAB, LEFT_DOCK, dockable, workspaceSurface } from "./shared";

export const UiDocumentStructureComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "ui.document.structure",
    title: "UI Structure",
    debugSource: "src/editors/ui-document/UiDocumentStructureDock.tsx",
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
);

export const UiDocumentEditorComponent = defineEditorComponent<UiDocumentEditorContext>()(
  workspaceSurface({
    id: "ui.document.editor",
    title: "UI Document Editor",
    debugSource: "src/editors/ui-document/UiDocumentEditor.tsx",
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
);

export const UI_DOCUMENT_COMPONENTS = [
  UiDocumentStructureComponent,
  UiDocumentEditorComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
