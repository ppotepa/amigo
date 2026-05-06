import type {
  EditorSceneHierarchyDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";

export type UiDocumentEditorTab = "tree" | "palette" | "templates";

export type UiDocumentEditorTarget = {
  modId: string;
  sceneId: string;
  entityId: string;
  componentIndex: number;
};

export type UiNodeCreateKind =
  | "column"
  | "row"
  | "panel"
  | "stack"
  | "spacer"
  | "text"
  | "button"
  | "image"
  | "progress-bar";

export type UiNodePaletteCategory = "layout" | "display" | "controls" | "feedback";

export type UiNodePaletteItem = {
  kind: UiNodeCreateKind;
  label: string;
  description: string;
  category: UiNodePaletteCategory;
  enabled: boolean;
  disabledReason?: string;
};

export type UiTemplateKind =
  | "empty-document"
  | "vertical-menu"
  | "button-row"
  | "health-bar"
  | "ammo-counter"
  | "dialogue-box"
  | "options-group"
  | "inventory-slot"
  | "notification-toast";

export type UiTemplateDefinition = {
  kind: UiTemplateKind;
  label: string;
  description: string;
  category: "document" | "menu" | "hud" | "dialogue" | "inventory" | "feedback";
  enabled: boolean;
  disabledReason?: string;
  previewLabel: string;
};

export type AddUiNodeDraft = {
  parentPath: string;
  kind: UiNodeCreateKind;
  id: string;
  label: string;
  text: string;
};

export type AddUiDocumentDraft = {
  name: string;
  entityId: string;
  target: "screen-space";
  viewportWidth: number;
  viewportHeight: number;
  template: UiTemplateKind;
};

export type AddUiTemplateDraft = {
  parentPath: string;
  template: UiTemplateKind;
  idPrefix: string;
};

export type UiDocumentEditorModel = {
  target: UiDocumentEditorTarget | null;
  document: EditorUiDocumentDto | null;
  selectedNode: EditorUiNodeDto | null;
  hierarchy: EditorSceneHierarchyDto | null;
};
