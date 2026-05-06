import { useState } from "react";
import type React from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import type { ContextTreeNode } from "./contextDockTypes";
import { ContextRow } from "./ContextRow";

export function ContextTree({ nodes }: { nodes: ContextTreeNode[] }) {
  if (!nodes.length) {
    return <p className="muted workspace-note">Nothing to show.</p>;
  }

  return (
    <div className="context-tree">
      {nodes.map((node) => (
        <ContextTreeItem key={node.id} node={node} depth={0} />
      ))}
    </div>
  );
}

function ContextTreeItem({
  depth,
  node,
}: {
  depth: number;
  node: ContextTreeNode;
}) {
  const hasChildren = Boolean(node.children?.length);
  const [expanded, setExpanded] = useState(node.defaultExpanded ?? depth < 1);

  return (
    <div className="context-tree-item">
      <div
        className={`context-tree-row ${hasChildren ? "context-tree-row-group" : "context-tree-row-leaf"}`}
        style={{ "--context-tree-depth": depth } as React.CSSProperties}
      >
        {hasChildren ? (
          <button
            className="context-tree-toggle"
            type="button"
            aria-label={expanded ? "Collapse" : "Expand"}
            onClick={() => setExpanded((current) => !current)}
          >
            {expanded ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
          </button>
        ) : (
          <span className="context-tree-spacer" />
        )}
        <ContextRow
          className={hasChildren ? "category-row" : undefined}
          actions={node.actions}
          badge={node.badge}
          icon={node.icon}
          selected={node.selected}
          subtitle={node.subtitle}
          title={node.title}
          tone={node.selected ? "cyan" : "default"}
          onClick={node.onSelect}
        />
      </div>

      {hasChildren && expanded ? (
        <div className="context-tree-children">
          {node.children?.map((child) => (
            <ContextTreeItem key={child.id} node={child} depth={depth + 1} />
          ))}
        </div>
      ) : null}
    </div>
  );
}
