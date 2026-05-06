import { Image, MousePointerClick, SlidersHorizontal } from "lucide-react";
import { useState } from "react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import {
  uiPreviewChildLayoutClass,
  uiPreviewHasVisibleChildren,
  uiPreviewNodeChrome,
  type UiPreviewMode,
} from "./uiPreviewRenderModel";

export function UiDocumentPreviewRenderer({
  document,
  mode,
  selectedPath,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  mode: UiPreviewMode;
  selectedPath: string | null;
  onSelectNode: (nodePath: string) => void;
}) {
  const [hoveredPath, setHoveredPath] = useState<string | null>(null);

  return (
    <div
      className={`ui-preview-renderer ui-preview-renderer-${mode}`}
      onMouseLeave={() => setHoveredPath(null)}
    >
      <UiPreviewNode
        mode={mode}
        node={document.root}
        selectedPath={selectedPath}
        hoveredPath={hoveredPath}
        onHoverNode={setHoveredPath}
        onSelectNode={onSelectNode}
      />
    </div>
  );
}

function UiPreviewNode({
  mode,
  node,
  selectedPath,
  hoveredPath,
  onHoverNode,
  onSelectNode,
}: {
  mode: UiPreviewMode;
  node: EditorUiNodeDto;
  selectedPath: string | null;
  hoveredPath: string | null;
  onHoverNode: (nodePath: string | null) => void;
  onSelectNode: (nodePath: string) => void;
}) {
  const chrome = uiPreviewNodeChrome(node, mode);
  const selected = selectedPath === node.path;
  const hovered = hoveredPath === node.path;
  const hasChildren = node.children.length > 0;
  const hasVisibleChildren = uiPreviewHasVisibleChildren(node);

  const className = [
    chrome.containerClassName,
    selected ? "selected" : "",
    hovered ? "hovered" : "",
    hasChildren ? "has-children" : "empty",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      className={className}
      style={chrome.inlineStyle}
      data-node-path={node.path}
      data-node-kind={node.kind}
      role="button"
      tabIndex={0}
      onClick={(event) => {
        event.stopPropagation();
        onSelectNode(node.path);
      }}
      onFocus={() => onHoverNode(node.path)}
      onMouseEnter={(event) => {
        event.stopPropagation();
        onHoverNode(node.path);
      }}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelectNode(node.path);
        }
      }}
    >
      {mode !== "preview" ? <UiPreviewNodeBadge node={node} selected={selected} /> : null}

      <div className={chrome.contentClassName}>
        <UiPreviewNodeContent node={node} text={chrome.text} mode={mode} />

        {chrome.isContainer ? (
          <div className={`ui-preview-children ${uiPreviewChildLayoutClass(node.kind)}`}>
            {hasVisibleChildren ? (
              node.children
                .filter((child) => child.visible !== false || mode !== "preview")
                .map((child) => (
                  <UiPreviewNode
                    key={child.path}
                    mode={mode}
                    node={child}
                    selectedPath={selectedPath}
                    hoveredPath={hoveredPath}
                    onHoverNode={onHoverNode}
                    onSelectNode={onSelectNode}
                  />
                ))
            ) : (
              <div className="ui-preview-empty-container">Empty {node.kind}</div>
            )}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function UiPreviewNodeContent({
  mode,
  node,
  text,
}: {
  mode: UiPreviewMode;
  node: EditorUiNodeDto;
  text: string;
}) {
  switch (node.kind) {
    case "text":
      return <span className="ui-preview-text">{text}</span>;

    case "button":
      return (
        <span className="ui-preview-button-face">
          <MousePointerClick size={12} />
          {text}
        </span>
      );

    case "progress-bar":
      return (
        <span className="ui-preview-progress">
          <span className="ui-preview-progress-fill" />
          <em>{text}</em>
        </span>
      );

    case "slider":
      return (
        <span className="ui-preview-slider">
          <span className="ui-preview-slider-track" />
          <span className="ui-preview-slider-thumb" />
        </span>
      );

    case "toggle":
      return (
        <span className="ui-preview-toggle">
          <span className="ui-preview-toggle-dot" />
          {text}
        </span>
      );

    case "dropdown":
    case "option-set":
      return (
        <span className="ui-preview-control-placeholder">
          <SlidersHorizontal size={12} />
          {text}
        </span>
      );

    case "spacer":
      return mode === "preview" ? null : <span className="ui-preview-spacer-label">Spacer</span>;

    case "unknown":
      return <span className="ui-preview-unknown">Unknown node</span>;

    case "panel":
    case "group-box":
    case "row":
    case "column":
    case "stack":
    case "tab-view":
      return null;

    default:
      return (
        <span className="ui-preview-image-placeholder">
          <Image size={14} />
          {text}
        </span>
      );
  }
}

function UiPreviewNodeBadge({
  node,
  selected,
}: {
  node: EditorUiNodeDto;
  selected: boolean;
}) {
  return (
    <span className={`ui-preview-node-badge ${selected ? "selected" : ""}`}>
      <strong>{node.id}</strong>
      <em>{node.kind}</em>
    </span>
  );
}
