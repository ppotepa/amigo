import type { CSSProperties, ReactNode } from "react";

export type TreeNodeTone = "root" | "folder" | "group" | "item" | "meta";

export type TreeViewNode = {
  key: string;
  children?: TreeViewNode[];
};

export type TreeViewRowRenderContext<TNode extends TreeViewNode> = {
  depth: number;
  expanded: boolean;
  hasChildren: boolean;
  node: TNode;
  toggle: () => void;
};

export function TreeView<TNode extends TreeViewNode>({
  nodes,
  expandedKeys,
  onToggle,
  renderNode,
}: {
  nodes: TNode[];
  expandedKeys: Set<string>;
  onToggle: (key: string) => void;
  renderNode: (context: TreeViewRowRenderContext<TNode>) => ReactNode;
}) {
  return (
    <div className="tree-view">
      {nodes.map((node) => (
        <TreeViewBranch key={node.key} depth={0} expandedKeys={expandedKeys} node={node} onToggle={onToggle} renderNode={renderNode} />
      ))}
    </div>
  );
}

function TreeViewBranch<TNode extends TreeViewNode>({
  node,
  depth,
  expandedKeys,
  onToggle,
  renderNode,
}: {
  node: TNode;
  depth: number;
  expandedKeys: Set<string>;
  onToggle: (key: string) => void;
  renderNode: (context: TreeViewRowRenderContext<TNode>) => ReactNode;
}) {
  const children = (node.children ?? []) as TNode[];
  const hasChildren = children.length > 0;
  const expanded = expandedKeys.has(node.key);

  return (
    <div className="tree-view-branch">
      {renderNode({
        depth,
        expanded,
        hasChildren,
        node,
        toggle: () => onToggle(node.key),
      })}
      {hasChildren && expanded ? (
        <div className="tree-view-children">
          {children.map((child) => (
            <TreeViewBranch key={child.key} depth={depth + 1} expandedKeys={expandedKeys} node={child} onToggle={onToggle} renderNode={renderNode} />
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function treeRowStyle(depth: number): CSSProperties {
  return { "--tree-depth": depth } as CSSProperties;
}
