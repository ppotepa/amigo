import type { EditorUiDocumentDto } from "../../api/dto";
import { TreeView, useTreeExpansion } from "../../ui/tree";
import { uiDocumentTreeAdapter } from "./uiDocumentTreeAdapter";

// @codemap anchor:ui-document-tree-panel domain:ui-document role:tree priority:P1 layer:app tags:tree,shared-tree,yaml-driven
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
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: uiDocumentTreeAdapter,
    nodes: [document.root],
    selectedId: selectedPath,
  });

  return (
    <section className="ui-document-panel ui-document-tree-panel">
      <header>
        <h3>Tree</h3>
        <span>{document.entityName}</span>
      </header>

      <TreeView
        actions={{
          onAddChild: (node) => onAddChild(node.path),
          onOpen: (node) => onOpenNode?.(node.path),
          onSelect: (node) => onSelectNode(node.path),
        }}
        adapter={uiDocumentTreeAdapter}
        className="ui-document-tree"
        expandedIds={expandedIds}
        nodes={[document.root]}
        onToggle={toggleExpanded}
        preset="outline"
        selectedId={selectedPath}
      />
    </section>
  );
}
