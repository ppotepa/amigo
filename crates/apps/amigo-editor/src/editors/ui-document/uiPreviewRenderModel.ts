import type { CSSProperties } from "react";
import type { EditorUiNodeDto, EditorUiNodeKindDto, EditorUiNodeStyleDto } from "../../api/dto";

export type UiPreviewMode = "preview" | "layout" | "debug";

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

export function uiPreviewNodeChrome(
  node: EditorUiNodeDto,
  mode: UiPreviewMode,
): UiPreviewNodeChrome {
  const kindClass = normalizeKindClass(node.kind);
  const isContainer = uiPreviewNodeCanHaveChildren(node.kind);
  const isLeaf = !isContainer;
  const text = uiPreviewNodeText(node);

  return {
    containerClassName: [
      "ui-preview-node",
      `ui-preview-node-${kindClass}`,
      isContainer ? "ui-preview-node-container" : "ui-preview-node-leaf",
      node.visible === false ? "ui-preview-node-hidden" : "",
      node.enabled === false ? "ui-preview-node-disabled" : "",
      mode !== "preview" ? "ui-preview-node-debuggable" : "",
    ]
      .filter(Boolean)
      .join(" "),
    contentClassName: [
      "ui-preview-node-content",
      `ui-preview-node-content-${kindClass}`,
    ]
      .filter(Boolean)
      .join(" "),
    inlineStyle: uiPreviewInlineStyle(node.style, node.kind),
    label: node.label || node.id || node.kind,
    text,
    isContainer,
    isLeaf,
  };
}

export function uiPreviewNodeCanHaveChildren(kind: EditorUiNodeKindDto | string): boolean {
  return [
    "panel",
    "group-box",
    "row",
    "column",
    "stack",
    "tab-view",
  ].includes(kind);
}

export function normalizeKindClass(kind: string): string {
  return kind.replace(/[^a-z0-9]+/gi, "-").toLowerCase();
}

export function uiPreviewNodeText(node: EditorUiNodeDto): string {
  if (node.text?.trim()) {
    return node.text;
  }

  switch (node.kind) {
    case "text":
      return node.label || node.id || "Text";
    case "button":
      return node.label || node.id || "Button";
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
  if (!style) {
    return {};
  }

  const result: CSSProperties = {};

  if (style.width != null) {
    result.width = style.width;
  }

  if (style.height != null) {
    result.height = style.height;
  }

  if (style.fontSize != null) {
    result.fontSize = style.fontSize;
  }

  if (style.color) {
    result.color = style.color;
  }

  if (style.background) {
    result.background = style.background;
  }

  if (style.borderColor) {
    result.borderColor = style.borderColor;
  }

  if (style.borderWidth != null) {
    result.borderWidth = style.borderWidth;
    result.borderStyle = "solid";
  }

  if (style.borderRadius != null) {
    result.borderRadius = style.borderRadius;
  }

  if (style.padding != null) {
    result.padding = style.padding;
  }

  if (style.gap != null) {
    result.gap = style.gap;
  }

  if (style.left != null || style.top != null) {
    result.position = "absolute";
    if (style.left != null) result.left = style.left;
    if (style.top != null) result.top = style.top;
  }

  if (kind === "spacer") {
    result.minHeight = style.height ?? 16;
    result.minWidth = style.width ?? 16;
  }

  return result;
}

export function uiPreviewChildLayoutClass(kind: EditorUiNodeKindDto | string): string {
  switch (kind) {
    case "row":
      return "ui-preview-children-row";
    case "stack":
      return "ui-preview-children-stack";
    case "panel":
    case "group-box":
    case "column":
    default:
      return "ui-preview-children-column";
  }
}

export function uiPreviewHasVisibleChildren(node: EditorUiNodeDto): boolean {
  return node.children.some((child) => child.visible !== false);
}
