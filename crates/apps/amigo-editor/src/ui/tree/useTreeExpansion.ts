import { useEffect, useMemo, useState } from "react";
import type { TreeNodeAdapter } from "./treeTypes";

// @codemap anchor:shared-tree-expansion-hook domain:workspace role:tree-expansion priority:P1 layer:app tags:tree,expanded-state
export function useTreeExpansion<TNode>({
  adapter,
  autoExpandRoots = true,
  nodes,
  selectedId,
}: {
  adapter: TreeNodeAdapter<TNode>;
  autoExpandRoots?: boolean;
  nodes: readonly TNode[];
  selectedId?: string | null;
}) {
  const initialExpandedIds = useMemo(
    () => collectDefaultExpandedIds(nodes, adapter, autoExpandRoots),
    [adapter, autoExpandRoots, nodes],
  );
  const [expandedIds, setExpandedIds] = useState<Set<string>>(() => initialExpandedIds);

  useEffect(() => {
    setExpandedIds((current) => {
      const index = indexTree(nodes, adapter);
      const next = new Set([...current].filter((id) => index.ids.has(id)));

      if (autoExpandRoots) {
        nodes.forEach((node) => next.add(adapter.getId(node)));
      }

      if (selectedId) {
        let parent = index.parentById.get(selectedId);
        while (parent) {
          next.add(parent);
          parent = index.parentById.get(parent);
        }
      }

      return setsEqual(current, next) ? current : next;
    });
  }, [adapter, autoExpandRoots, nodes, selectedId]);

  function toggleExpanded(id: string) {
    setExpandedIds((current) => {
      const next = new Set(current);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  return { expandedIds, setExpandedIds, toggleExpanded };
}

function collectDefaultExpandedIds<TNode>(
  nodes: readonly TNode[],
  adapter: TreeNodeAdapter<TNode>,
  autoExpandRoots: boolean,
): Set<string> {
  const ids = new Set<string>();

  function visit(node: TNode, depth: number) {
    const id = adapter.getId(node);
    const children = adapter.getChildren(node);
    if ((autoExpandRoots && depth === 0) || children.length > 0) {
      ids.add(id);
    }
    children.forEach((child) => visit(child, depth + 1));
  }

  nodes.forEach((node) => visit(node, 0));
  return ids;
}

function indexTree<TNode>(nodes: readonly TNode[], adapter: TreeNodeAdapter<TNode>) {
  const ids = new Set<string>();
  const parentById = new Map<string, string>();

  function visit(node: TNode, parentId: string | null) {
    const id = adapter.getId(node);
    ids.add(id);
    if (parentId) parentById.set(id, parentId);
    adapter.getChildren(node).forEach((child) => visit(child, id));
  }

  nodes.forEach((node) => visit(node, null));
  return { ids, parentById };
}

function setsEqual(left: Set<string>, right: Set<string>) {
  if (left.size !== right.size) return false;
  for (const value of left) {
    if (!right.has(value)) return false;
  }
  return true;
}
