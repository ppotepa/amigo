import { useMemo } from "react";
import type { EditorTargetIntent } from "../../editor-targets/editorTargetTypes";
import { TreeView, useTreeExpansion } from "../../ui/tree";
import { projectTreeAdapter } from "./projectTreeAdapter";
import {
  normalizeProjectTreeNode,
  projectNodeMatchesSearch,
  type ProjectExplorerTreeNode,
} from "./projectTreeModel";

// @codemap anchor:project-explorer-shared-tree domain:project role:tree priority:P1 layer:app tags:tree,project,shared-tree,editor-target
export function ProjectExplorerTree({
  collapsedSearch,
  node,
  onActivateNode,
  onCreateExpectedFolder,
  onOpenContextMenu,
  selectedFilePath,
  selectedNodeId,
  selectedSceneId,
}: {
  collapsedSearch: string;
  node: ProjectExplorerTreeNode;
  onActivateNode: (node: ProjectExplorerTreeNode, intent: EditorTargetIntent) => void;
  onCreateExpectedFolder?: (expectedPath: string) => Promise<void>;
  onOpenContextMenu: (node: ProjectExplorerTreeNode, x: number, y: number) => void;
  selectedFilePath: string | null;
  selectedNodeId: string | null;
  selectedSceneId: string | null;
}) {
  const tree = useMemo(() => filterProjectTree(node, collapsedSearch) ?? node, [node, collapsedSearch]);
  const resolvedSelectedId = selectedNodeId ?? selectedSceneId ?? selectedFilePath ?? null;
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: projectTreeAdapter,
    nodes: [tree],
    selectedId: resolvedSelectedId,
  });

  return (
    <TreeView
      actions={{
        onAction: (actionId, target) => {
          if (actionId === "open") onActivateNode(target, "open");
          if (actionId === "createExpectedFolder" && target.expectedPath) {
            void onCreateExpectedFolder?.(target.expectedPath);
          }
          if (actionId === "copyPath") {
            void navigator.clipboard.writeText(target.path ?? target.expectedPath ?? "");
          }
        },
        onContextMenu: (target, event) => {
          event.preventDefault();
          onActivateNode(target, "contextMenu");
          onOpenContextMenu(target, event.clientX, event.clientY);
        },
        onOpen: (target) => onActivateNode(target, "open"),
        onSelect: (target) => onActivateNode(target, "select"),
      }}
      adapter={projectTreeAdapter}
      className="project-explorer-tree"
      expandedIds={expandedIds}
      nodes={[tree]}
      onToggle={toggleExpanded}
      preset="outline"
      selectedId={resolvedSelectedId}
    />
  );
}

function filterProjectTree(
  node: ProjectExplorerTreeNode,
  search: string,
): ProjectExplorerTreeNode | null {
  const query = search.trim().toLowerCase();
  const children = (node.children ?? [])
    .map((child) => filterProjectTree(child, search))
    .filter((child): child is ProjectExplorerTreeNode => Boolean(child));

  if (!query || projectNodeMatchesSearch(node, query) || children.length > 0) {
    return normalizeProjectTreeNode(node, children.map((child) => normalizeProjectTreeNode(child)));
  }

  return null;
}
