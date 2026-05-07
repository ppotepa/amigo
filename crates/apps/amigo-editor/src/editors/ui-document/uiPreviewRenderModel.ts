import type { CSSProperties } from "react";
import type { EditorUiNodeDto, EditorUiNodeKindDto, EditorUiNodeStyleDto } from "../../api/dto";
import { uiNodeCapabilitiesForKind } from "./uiNodeCapabilities";

// @codemap anchor:ui-document-preview-mode-model domain:ui-document role:model priority:P1 layer:app tags:preview,simple,realtime
export type UiPreviewMode = "simple" | "realtime";

export type UiScreenLink = {
  fromPath: string;
  fromLabel: string;
  to: string;
  event: string;
};

export type UiPreviewNodeChrome = {
  containerClassName: string;
  contentClassName: string;
  inlineStyle: CSSProperties;
  label: string;
  text: string;
  isContainer: boolean;
  isLeaf: boolean;
};

export const DEFAULT_UI_ARTBOARD_WIDTH = 1280;
export const DEFAULT_UI_ARTBOARD_HEIGHT = 720;

export function uiPreviewNodeChrome(node: EditorUiNodeDto, mode: UiPreviewMode): UiPreviewNodeChrome {
  const kindClass = normalizeKindClass(node.kind);
  const isContainer = uiPreviewNodeCanHaveChildren(node.kind);
  const text = uiPreviewNodeText(node);

  return {
    containerClassName: [
      "ui-preview-node",
      `ui-preview-node-${kindClass}`,
      isContainer ? "ui-preview-node-container" : "ui-preview-node-leaf",
      node.visible === false ? "ui-preview-node-hidden" : "",
      node.enabled === false ? "ui-preview-node-disabled" : "",
      mode === "simple" ? "ui-preview-node-debuggable" : "",
    ]
      .filter(Boolean)
      .join(" "),
    contentClassName: ["ui-preview-node-content", `ui-preview-node-content-${kindClass}`]
      .filter(Boolean)
      .join(" "),
    inlineStyle: uiPreviewInlineStyle(node.style, node.kind),
    label: node.label || node.id || node.kind,
    text,
    isContainer,
    isLeaf: !isContainer,
  };
}

export function uiPreviewNodeCanHaveChildren(kind: EditorUiNodeKindDto | string): boolean {
  return uiNodeCapabilitiesForKind(kind).canHaveChildren;
}

export function normalizeKindClass(kind: string): string {
  return kind.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
}

export function uiPreviewNodeText(node: EditorUiNodeDto): string {
  if (node.text?.trim()) return node.text;

  switch (node.kind) {
    case "text":
      return node.label || node.id || "Text";
    case "button":
      return node.label || node.id || "Button";
    case "image":
      return node.label || node.id || "Image";
    case "progress-bar":
      return "Progress";
    case "slider":
      return "Slider";
    case "toggle":
      return node.label || node.id || "Toggle";
    case "dropdown":
      return node.label || node.id || "Dropdown";
    case "option-set":
      return node.label || node.id || "Option Set";
    case "spacer":
      return "Spacer";
    default:
      return node.label || node.id || node.kind;
  }
}

export function uiPreviewInlineStyle(
  style: EditorUiNodeStyleDto | null | undefined,
  kind: EditorUiNodeKindDto,
): CSSProperties {
  if (!style) return {};

  const result: CSSProperties = {};

  if (style.width != null) result.width = style.width;
  if (style.height != null) result.height = style.height;
  if (style.fontSize != null) result.fontSize = style.fontSize;
  if (style.color) result.color = style.color;
  if (style.background) result.background = style.background;
  if (style.borderColor) result.borderColor = style.borderColor;
  if (style.borderWidth != null) {
    result.borderWidth = style.borderWidth;
    result.borderStyle = "solid";
  }
  if (style.borderRadius != null) result.borderRadius = style.borderRadius;
  if (style.padding != null) result.padding = style.padding;
  if (style.gap != null) result.gap = style.gap;

  if (kind === "spacer" && result.width == null && result.height == null) {
    result.width = 24;
    result.height = 18;
  }

  return result;
}

export function uiPreviewChildLayoutClass(kind: string): string {
  switch (kind) {
    case "row":
      return "ui-preview-children-row";
    case "stack":
    case "tab-view":
      return "ui-preview-children-stack";
    case "column":
    case "panel":
    case "group-box":
    default:
      return "ui-preview-children-column";
  }
}

export function uiPreviewHasVisibleChildren(node: EditorUiNodeDto): boolean {
  return node.children.some((child) => child.visible !== false);
}

export function collectUiScreenLinks(root: EditorUiNodeDto): UiScreenLink[] {
  const result: UiScreenLink[] = [];

  function visit(node: EditorUiNodeDto) {
    if (node.actionEvent?.trim() && node.actionTarget?.trim()) {
      result.push({
        fromPath: node.path,
        fromLabel: node.label || node.text || node.id,
        event: node.actionEvent,
        to: node.actionTarget,
      });
    }

    node.children.forEach(visit);
  }

  visit(root);
  return result;
}

// @codemap anchor:ui-document-preview-scope-model domain:ui-document role:model priority:P1 layer:app tags:focusPath,scoped-view,yaml-driven
export function resolveUiDocumentPreviewRoot(
  root: EditorUiNodeDto,
  focusPath: string | null | undefined,
): EditorUiNodeDto {
  if (focusPath) {
    return findUiPreviewNode(root, focusPath) ?? root;
  }

  return defaultUiDocumentPreviewRoot(root);
}

export function defaultUiDocumentPreviewRoot(root: EditorUiNodeDto): EditorUiNodeDto {
  return root.children[0] ?? root;
}

export function findUiPreviewNode(
  root: EditorUiNodeDto,
  path: string | null | undefined,
): EditorUiNodeDto | null {
  if (!path) return null;
  if (root.path === path) return root;

  for (const child of root.children) {
    const found = findUiPreviewNode(child, path);
    if (found) return found;
  }

  return null;
}
