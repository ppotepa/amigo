import { useMemo, useState } from "react";
import type { CSSProperties } from "react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import {
  collectUiScreenLinks,
  resolveUiDocumentPreviewRoot,
  uiPreviewChildLayoutClass,
  uiPreviewHasVisibleChildren,
  uiPreviewNodeChrome,
  uiPreviewNodeText,
  type UiPreviewMode,
} from "./uiPreviewRenderModel";

export function UiDocumentPreviewRenderer({
  document,
  focusPath,
  mode,
  selectedPath,
  revision,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  focusPath: string | null;
  mode: UiPreviewMode;
  selectedPath: string | null;
  revision: number;
  onSelectNode: (nodePath: string) => void;
}) {
  const [hoveredPath, setHoveredPath] = useState<string | null>(null);
  const previewRoot = useMemo(
    () => resolveUiDocumentPreviewRoot(document.root, focusPath),
    [document.root, focusPath],
  );
  const links = useMemo(() => collectUiScreenLinks(document.root), [document.root]);

  if (mode === "realtime") {
    return (
      <RealtimePreview
        key={revision}
        document={document}
        screen={previewRoot}
        selectedPath={selectedPath}
        onSelectNode={onSelectNode}
      />
    );
  }

  return (
    <div
      className="ui-preview-renderer ui-preview-renderer-simple"
      onMouseLeave={() => setHoveredPath(null)}
    >
      <div className="ui-simple-screen-frame">
        <UiSimpleNode
          node={previewRoot}
          selectedPath={selectedPath}
          hoveredPath={hoveredPath}
          onHoverNode={setHoveredPath}
          onSelectNode={onSelectNode}
        />
      </div>

      <ScreenLinksPanel links={links} />
    </div>
  );
}

// @codemap anchor:ui-document-simple-preview-renderer domain:ui-document role:renderer priority:P1 layer:app tags:simple,tree,true-state
function UiSimpleNode({
  node,
  selectedPath,
  hoveredPath,
  onHoverNode,
  onSelectNode,
}: {
  node: EditorUiNodeDto;
  selectedPath: string | null;
  hoveredPath: string | null;
  onHoverNode: (nodePath: string | null) => void;
  onSelectNode: (nodePath: string) => void;
}) {
  if (node.visible === false) return null;

  const chrome = uiPreviewNodeChrome(node, "simple");
  const selected = selectedPath === node.path;
  const hovered = hoveredPath === node.path;
  const hasChildren = node.children.length > 0;
  const hasVisibleChildren = uiPreviewHasVisibleChildren(node);
  const className = [
    chrome.containerClassName,
    "ui-simple-node",
    `ui-simple-node-${node.kind.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`,
    selected ? "selected" : "",
    hovered ? "hovered" : "",
    hasChildren ? "has-children" : "empty",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      className={className}
      style={simpleNodeStyle(node)}
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
      <UiSimpleNodeContent node={node} />

      {chrome.isContainer ? (
        <div className={`ui-preview-children ${uiPreviewChildLayoutClass(node.kind)}`}>
          {hasVisibleChildren ? (
            node.children.map((child) => (
              <UiSimpleNode
                key={child.path}
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
  );
}

function UiSimpleNodeContent({ node }: { node: EditorUiNodeDto }) {
  const text = uiPreviewNodeText(node);

  switch (node.kind) {
    case "text":
      return <div className="ui-simple-text">{text}</div>;
    case "button":
      return (
        <div className="ui-simple-button-face">
          <span>{text}</span>
        </div>
      );
    case "image":
      return <div className="ui-simple-image-placeholder">{text}</div>;
    case "spacer":
      return <div className="ui-simple-spacer" />;
    default:
      return null;
  }
}

function simpleNodeStyle(node: EditorUiNodeDto): CSSProperties {
  const style = node.style ?? {};
  const result: CSSProperties = {};

  if (style.width != null) result.width = style.width;
  if (style.height != null) result.height = style.height;
  if (style.fontSize != null) result.fontSize = style.fontSize;
  if (style.padding != null) result.padding = style.padding;
  if (style.gap != null) result.gap = style.gap;

  return result;
}

function ScreenLinksPanel({ links }: { links: ReturnType<typeof collectUiScreenLinks> }) {
  if (!links.length) return null;

  return (
    <aside className="ui-screen-links-panel">
      <header>Screen Links</header>
      <div className="ui-screen-links-root">UiDocument</div>
      {links.map((link) => (
        <div key={`${link.fromPath}:${link.to}`} className="ui-screen-link-row">
          <span>{link.fromLabel}</span>
          <strong>{link.event}</strong>
          <em>{link.to}</em>
        </div>
      ))}
    </aside>
  );
}

// @codemap anchor:ui-document-realtime-preview-shell domain:ui-document role:renderer priority:P1 layer:app tags:realtime,runtime-preview
function RealtimePreview({
  document,
  screen,
  selectedPath,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  screen: EditorUiNodeDto;
  selectedPath: string | null;
  onSelectNode: (nodePath: string) => void;
}) {
  const nodes = flattenVisibleNodes(screen);
  const menuNodes = nodes.filter((node) => node.kind === "button");
  const titleNode = nodes.find((node) => node.kind === "text");
  const title = titleNode?.text || titleNode?.label || document.entityName || "UI Preview";

  return (
    <div className="ui-realtime-preview" data-document={document.entityName}>
      <div className="ui-realtime-badge">Runtime Preview</div>
      <div className="ui-realtime-backdrop" />
      <div className="ui-realtime-content">
        <h1>{title.toUpperCase()}</h1>
        <nav className="ui-realtime-menu">
          {menuNodes.map((node) => {
            const selected = node.path === selectedPath;
            return (
              <button
                key={node.path}
                type="button"
                className={selected ? "selected" : ""}
                onClick={() => onSelectNode(node.path)}
              >
                {uiPreviewNodeText(node).toUpperCase()}
              </button>
            );
          })}
        </nav>
      </div>
    </div>
  );
}

function flattenVisibleNodes(node: EditorUiNodeDto): EditorUiNodeDto[] {
  if (node.visible === false) return [];
  return [node, ...node.children.flatMap(flattenVisibleNodes)];
}
