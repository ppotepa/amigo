import type {
  EditorSceneHierarchyDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";
import type {
  AddUiDocumentDraft,
  AddUiNodeDraft,
  AddUiTemplateDraft,
  UiDocumentEditorTarget,
  UiNodeCreateKind,
} from "./uiDocumentEditorTypes";
import { uiNodeCapabilitiesForKind } from "./uiNodeCapabilities";

export function uiDocumentTargetKey(target: UiDocumentEditorTarget): string {
  return `${target.sceneId}:${target.entityId}:${target.componentIndex}`;
}

export function findUiDocument(
  hierarchy: EditorSceneHierarchyDto | null | undefined,
  target: UiDocumentEditorTarget | null | undefined,
): EditorUiDocumentDto | null {
  if (!hierarchy || !target) return null;

  return (
    hierarchy.uiDocuments.find(
      (document) =>
        document.entityId === target.entityId &&
        document.componentIndex === target.componentIndex,
    ) ?? null
  );
}

export function findUiNode(
  root: EditorUiNodeDto | null | undefined,
  path: string | null | undefined,
): EditorUiNodeDto | null {
  if (!root || !path) return null;
  if (root.path === path) return root;

  for (const child of root.children) {
    const found = findUiNode(child, path);
    if (found) return found;
  }

  return null;
}

export function flattenUiNodes(root: EditorUiNodeDto | null | undefined): EditorUiNodeDto[] {
  if (!root) return [];
  return [root, ...root.children.flatMap(flattenUiNodes)];
}

export function canHaveChildren(kind: string): boolean {
  return uiNodeCapabilitiesForKind(kind).canHaveChildren;
}

export function getSiblingInfo(
  root: EditorUiNodeDto | null | undefined,
  nodePath: string | null | undefined,
): { parentPath: string; index: number; count: number } | null {
  if (!root || !nodePath || root.path === nodePath) return null;

  const parentPath = nodePath.split(".").slice(0, -1).join(".");
  const parent = findUiNode(root, parentPath);
  if (!parent) return null;

  const index = parent.children.findIndex((child) => child.path === nodePath);
  if (index < 0) return null;

  return {
    parentPath,
    index,
    count: parent.children.length,
  };
}

export function defaultNodeId(kind: UiNodeCreateKind): string {
  switch (kind) {
    case "text":
      return "new-text";
    case "button":
      return "new-button";
    case "panel":
      return "new-panel";
    case "column":
      return "new-column";
    case "row":
      return "new-row";
    case "stack":
      return "new-stack";
    case "spacer":
      return "new-spacer";
    case "image":
      return "new-image";
    case "progress-bar":
      return "new-progress-bar";
    default:
      return "new-node";
  }
}

export function defaultNodeText(kind: UiNodeCreateKind): string {
  switch (kind) {
    case "text":
      return "New Text";
    case "button":
      return "New Button";
    case "progress-bar":
      return "100 / 100";
    default:
      return "";
  }
}

export function createDefaultAddNodeDraft(
  parentPath: string,
  kind: UiNodeCreateKind = "button",
): AddUiNodeDraft {
  const id = defaultNodeId(kind);
  return {
    parentPath,
    kind,
    id,
    label: labelFromId(id),
    text: defaultNodeText(kind),
  };
}

export function createDefaultAddUiDocumentDraft(): AddUiDocumentDraft {
  return {
    name: "Main UI",
    entityId: "main-ui",
    target: "screen-space",
    viewportWidth: 1280,
    viewportHeight: 720,
    template: "empty-document",
  };
}

export function createDefaultAddTemplateDraft(parentPath: string): AddUiTemplateDraft {
  return {
    parentPath,
    template: "vertical-menu",
    idPrefix: "menu",
  };
}

export function labelFromId(id: string): string {
  return id
    .split("-")
    .filter(Boolean)
    .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
    .join(" ");
}

export function isValidUiNodeId(id: string): boolean {
  return /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/.test(id);
}

export function validateAddNodeDraft(draft: AddUiNodeDraft): string | null {
  if (!draft.parentPath.trim()) return "Parent path is required.";
  if (!isValidUiNodeId(draft.id)) return "Node id must use lowercase letters, numbers and dashes.";
  if ((draft.kind === "text" || draft.kind === "button") && !draft.text.trim()) {
    return "Text is required for Text and Button nodes.";
  }
  return null;
}

export function validateAddUiDocumentDraft(draft: AddUiDocumentDraft): string | null {
  if (!draft.name.trim()) return "Document name is required.";
  if (!isValidUiNodeId(draft.entityId)) return "Entity id must use lowercase letters, numbers and dashes.";
  if (draft.viewportWidth <= 0 || draft.viewportHeight <= 0) return "Viewport must be greater than zero.";
  return null;
}

export function validateAddTemplateDraft(draft: AddUiTemplateDraft): string | null {
  if (!draft.parentPath.trim()) return "Parent path is required.";
  if (!isValidUiNodeId(draft.idPrefix)) {
    return "Template id prefix must use lowercase letters, numbers and dashes.";
  }
  return null;
}
