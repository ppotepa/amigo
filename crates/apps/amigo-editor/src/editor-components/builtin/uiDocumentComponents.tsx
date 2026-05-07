import { UiDocumentEditor } from "../../editors/ui-document";
import type { EditorComponentDefinition } from "../componentTypes";
import { CENTER_TAB, workspaceSurface } from "./shared";

export const UI_DOCUMENT_COMPONENTS: EditorComponentDefinition[] = [
  workspaceSurface({
    id: "ui.document.editor",
    title: "UI Document Editor",
    category: "ui",
    domain: "ui",
    icon: "layout-template",
    description: "Inspect and edit UiDocument node trees.",
    placement: CENTER_TAB,
    defaultPlacement: CENTER_TAB,
    allowedPlacements: ["centerTab", "floatingPanel", "window"],
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
