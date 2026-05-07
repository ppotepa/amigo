import type { EditorUiNodeDto, EditorUiNodeKindDto } from "../../api/dto";
import type { UiNodeCreateKind } from "./uiDocumentEditorTypes";

// @codemap anchor:ui-node-capabilities domain:ui-document role:model priority:P1 layer:app tags:tree,add-node,capabilities
export type UiNodeCapabilities = {
  canHaveChildren: boolean;
  canAddChild: boolean;
  canEditText: boolean;
  canEditStyle: boolean;
  canEditLayout: boolean;
  canTrigger: boolean;
  canNavigate: boolean;
  canPickAsset: boolean;
  allowedChildren: UiNodeCreateKind[];
};

const LAYOUT_CHILDREN: UiNodeCreateKind[] = [
  "column",
  "row",
  "panel",
  "stack",
  "spacer",
  "text",
  "button",
  "image",
  "progress-bar",
];

const CONTROL_CHILDREN: UiNodeCreateKind[] = [];

const BASE_LEAF: UiNodeCapabilities = {
  canHaveChildren: false,
  canAddChild: false,
  canEditText: false,
  canEditStyle: true,
  canEditLayout: true,
  canTrigger: false,
  canNavigate: false,
  canPickAsset: false,
  allowedChildren: [],
};

const CONTAINER: UiNodeCapabilities = {
  canHaveChildren: true,
  canAddChild: true,
  canEditText: false,
  canEditStyle: true,
  canEditLayout: true,
  canTrigger: false,
  canNavigate: false,
  canPickAsset: false,
  allowedChildren: LAYOUT_CHILDREN,
};

export function uiNodeCapabilitiesForKind(
  kind: EditorUiNodeKindDto | UiNodeCreateKind | string,
): UiNodeCapabilities {
  switch (kind) {
    case "column":
    case "row":
    case "panel":
    case "group-box":
    case "stack":
    case "tab-view":
      return CONTAINER;

    case "text":
      return {
        ...BASE_LEAF,
        canEditText: true,
      };

    case "button":
      return {
        ...BASE_LEAF,
        canEditText: true,
        canTrigger: true,
        canNavigate: true,
        allowedChildren: CONTROL_CHILDREN,
      };

    case "image":
      return {
        ...BASE_LEAF,
        canPickAsset: true,
      };

    case "spacer":
      return {
        ...BASE_LEAF,
        canEditStyle: false,
      };

    case "progress-bar":
    case "slider":
    case "toggle":
    case "option-set":
    case "dropdown":
    case "color-picker-rgb":
    case "curve-editor":
      return BASE_LEAF;

    case "unknown":
    default:
      return {
        ...BASE_LEAF,
        canEditStyle: false,
        canEditLayout: false,
      };
  }
}

export function uiNodeCapabilities(node: EditorUiNodeDto | null | undefined): UiNodeCapabilities {
  return uiNodeCapabilitiesForKind(node?.kind ?? "unknown");
}

export function uiNodeCanHaveChildren(node: EditorUiNodeDto | null | undefined): boolean {
  return uiNodeCapabilities(node).canHaveChildren;
}

export function uiNodeCanAddChild(node: EditorUiNodeDto | null | undefined): boolean {
  return uiNodeCapabilities(node).canAddChild;
}

export function allowedUiChildrenForNode(node: EditorUiNodeDto | null | undefined): UiNodeCreateKind[] {
  return uiNodeCapabilities(node).allowedChildren;
}

export function firstAllowedUiChildKind(node: EditorUiNodeDto | null | undefined): UiNodeCreateKind | null {
  return allowedUiChildrenForNode(node)[0] ?? null;
}

export function uiNodeCannotHaveChildrenReason(node: EditorUiNodeDto | null | undefined): string {
  if (!node) return "Select a parent node first.";
  return `${node.label || node.id || node.kind} cannot contain child nodes.`;
}
