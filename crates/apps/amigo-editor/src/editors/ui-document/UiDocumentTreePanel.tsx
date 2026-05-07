import { useEffect, useState } from "react";
import { ChevronDown, ChevronRight, Plus } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import { uiNodeCanAddChild } from "./uiNodeCapabilities";
import { UiNodeKindIcon, uiNodeKindLabel } from "./uiNodeTreeIcons";

// @codemap anchor:ui-document-real-tree-panel domain:ui-document role:tree priority:P1 layer:app tags:yaml-driven,tree,double-click
export function UiDocumentTreePanel({
  document,
  selectedPath,
  onAddChild,
  onOpenNode,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  selectedPath: string | null;
  onAddChild: (parentPath: string) => void;
  onOpenNode?: (nodePath: string) => void;
  onSelectNode: (nodePath: string) => void;
}) {
  const [expandedPaths, setExpandedPaths] = useState<Set<string>>(() => new Set([document.root.path]));

  useEffect(() => {
    setExpandedPaths((current) => {
      const next = new Set(current);
      next.add(document.root.path);

      if (selectedPath) {
        for (const path of parentPathsFor(selectedPath)) {
          next.add(path);
        }
      }

      return next;
    });
  }, [document.root.path, selectedPath]);

  function toggleExpanded(nodePath: string) {
    setExpandedPaths((current) => {
      const next = new Set(current);
      if (next.has(nodePath)) {
        next.delete(nodePath);
      } else {
        next.add(nodePath);
      }

      return next;
    });
  }

  return (
    <section className="ui-document-panel ui-document-tree-panel">
      <header>
        <h3>Tree</h3>
        <span>{document.entityName}</span>
      </header>

      <div className="ui-document-tree">
        <UiNodeTreeRow
          depth={0}
          expandedPaths={expandedPaths}
          node={document.root}
          selectedPath={selectedPath}
          onAddChild={onAddChild}
          onOpenNode={onOpenNode}
          onSelectNode={onSelectNode}
          onToggleExpanded={toggleExpanded}
        />
      </div>
    </section>
  );
}

function UiNodeTreeRow({
  depth,
  expandedPaths,
  node,
  selectedPath,
  onAddChild,
  onOpenNode,
  onSelectNode,
  onToggleExpanded,
}: {
  depth: number;
  expandedPaths: Set<string>;
  node: EditorUiNodeDto;
  selectedPath: string | null;
  onAddChild: (parentPath: string) => void;
  onOpenNode?: (nodePath: string) => void;
  onSelectNode: (nodePath: string) => void;
  onToggleExpanded: (nodePath: string) => void;
}) {
  const hasChildren = node.children.length > 0;
  const canAddChild = uiNodeCanAddChild(node);
  const expanded = expandedPaths.has(node.path);
  const selected = selectedPath === node.path;

  return (
    <div className="ui-node-tree-row-wrap">
      <div
        className={`ui-node-tree-row ${selected ? "selected" : ""}`}
        style={{ paddingLeft: `${depth * 14 + 8}px` }}
      >
        {hasChildren ? (
          <button
            className="ui-node-tree-expander"
            aria-label={expanded ? "Collapse node" : "Expand node"}
            type="button"
            onClick={() => onToggleExpanded(node.path)}
          >
            {expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
          </button>
        ) : (
          <span className="ui-node-tree-expander" aria-hidden="true" />
        )}

        <button
          className="ui-node-tree-main"
          title="Single click selects. Double click opens focused view."
          type="button"
          onClick={() => onSelectNode(node.path)}
          onDoubleClick={() => onOpenNode?.(node.path)}
        >
          <UiNodeKindIcon kind={node.kind} />
          <span className="ui-node-tree-label">{node.label}</span>
          <em>{uiNodeKindLabel(node.kind)}</em>
        </button>

        {canAddChild ? (
          <button
            className="ui-node-tree-add"
            title="Add child node"
            type="button"
            onClick={() => onAddChild(node.path)}
          >
            <Plus size={13} />
          </button>
        ) : null}
      </div>

      {expanded
        ? node.children.map((child) => (
            <UiNodeTreeRow
              key={child.path}
              depth={depth + 1}
              expandedPaths={expandedPaths}
              node={child}
              selectedPath={selectedPath}
              onAddChild={onAddChild}
              onOpenNode={onOpenNode}
              onSelectNode={onSelectNode}
              onToggleExpanded={onToggleExpanded}
            />
          ))
        : null}
    </div>
  );
}

function parentPathsFor(path: string): string[] {
  const parts = path.split(".");
  const result: string[] = [];

  for (let index = 1; index < parts.length; index += 1) {
    result.push(parts.slice(0, index).join("."));
  }

  return result;
}
