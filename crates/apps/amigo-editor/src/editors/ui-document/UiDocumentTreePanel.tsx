import { ChevronDown, ChevronRight, Plus, Rows3 } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";

export function UiDocumentTreePanel({
  document,
  selectedPath,
  onAddChild,
  onSelectNode,
}: {
  document: EditorUiDocumentDto;
  selectedPath: string | null;
  onAddChild: (parentPath: string) => void;
  onSelectNode: (nodePath: string) => void;
}) {
  return (
    <section className="ui-document-panel ui-document-tree-panel">
      <header>
        <h3>Tree</h3>
        <span>{document.entityName}</span>
      </header>

      <div className="ui-document-tree">
        <UiNodeTreeRow
          depth={0}
          node={document.root}
          selectedPath={selectedPath}
          onAddChild={onAddChild}
          onSelectNode={onSelectNode}
        />
      </div>
    </section>
  );
}

function UiNodeTreeRow({
  depth,
  node,
  selectedPath,
  onAddChild,
  onSelectNode,
}: {
  depth: number;
  node: EditorUiNodeDto;
  selectedPath: string | null;
  onAddChild: (parentPath: string) => void;
  onSelectNode: (nodePath: string) => void;
}) {
  const hasChildren = node.children.length > 0;
  const selected = selectedPath === node.path;

  return (
    <div className="ui-node-tree-row-wrap">
      <div
        className={`ui-node-tree-row ${selected ? "selected" : ""}`}
        style={{ paddingLeft: `${depth * 14 + 8}px` }}
      >
        <button className="ui-node-tree-main" type="button" onClick={() => onSelectNode(node.path)}>
          <span className="ui-node-tree-expander">
            {hasChildren ? <ChevronDown size={13} /> : <ChevronRight size={13} />}
          </span>
          <Rows3 size={14} />
          <span className="ui-node-tree-label">{node.label}</span>
          <em>{node.kind}</em>
        </button>

        <button
          className="ui-node-tree-add"
          title="Add child node"
          type="button"
          onClick={() => onAddChild(node.path)}
        >
          <Plus size={13} />
        </button>
      </div>

      {node.children.map((child) => (
        <UiNodeTreeRow
          key={child.path}
          depth={depth + 1}
          node={child}
          selectedPath={selectedPath}
          onAddChild={onAddChild}
          onSelectNode={onSelectNode}
        />
      ))}
    </div>
  );
}
