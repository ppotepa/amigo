import type { ContextTreeNode } from "./contextDockTypes";
import { TreeView, useTreeExpansion, type TreeNodeAdapter, type TreeNodeCapabilities } from "../tree";

function contextTreeCapabilities(node: ContextTreeNode, context: { hasChildren: boolean }): TreeNodeCapabilities {
  return {
    canExpand: context.hasChildren,
    canSelect: Boolean(node.onSelect),
    canOpen: Boolean(node.onSelect),
    canAddChild: false,
    canRename: false,
    canDelete: false,
    canDrag: false,
    canDropOn: false,
  };
}

const contextTreeAdapter: TreeNodeAdapter<ContextTreeNode> = {
  getId: (node) => node.id,
  getLabel: (node) => node.title,
  getChildren: (node) => node.children ?? [],
  getIcon: (node) => node.icon,
  getMeta: (node) => node.subtitle,
  getActionSlot: (node) => node.actions,
  getBadges: (node) => [
    {
      label: node.badge,
      visible: Boolean(node.badge),
    },
  ],
  getClassName: (node) => node.selected ? "context-tree-node-selected" : null,
  getCapabilities: (node, context) => contextTreeCapabilities(node, context),
};

export function ContextTree({ nodes }: { nodes: ContextTreeNode[] }) {
  const selectedId = findSelectedNodeId(nodes);
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: contextTreeAdapter,
    nodes,
    selectedId,
  });

  if (!nodes.length) {
    return <p className="muted workspace-note">Nothing to show.</p>;
  }

  return (
    <TreeView
      actions={{
        onOpen: (node) => node.onSelect?.(),
        onSelect: (node) => node.onSelect?.(),
      }}
      adapter={contextTreeAdapter}
      className="context-tree"
      expandedIds={expandedIds}
      nodes={nodes}
      onToggle={toggleExpanded}
      preset="compact"
      selectedId={selectedId}
    />
  );
}

function findSelectedNodeId(nodes: readonly ContextTreeNode[]): string | null {
  for (const node of nodes) {
    if (node.selected) return node.id;
    const child = findSelectedNodeId(node.children ?? []);
    if (child) return child;
  }
  return null;
}
