import { Copy, MoveDown, MoveUp, Plus, Trash2 } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";

export function UiDocumentInspectorPanel({
  document,
  selectedNode,
  onAddChild,
  onDuplicate,
  onMoveDown,
  onMoveUp,
  onRemove,
}: {
  document: EditorUiDocumentDto;
  selectedNode: EditorUiNodeDto | null;
  onAddChild: () => void;
  onDuplicate: () => void;
  onMoveDown: () => void;
  onMoveUp: () => void;
  onRemove: () => void;
}) {
  return (
    <section className="ui-document-panel ui-document-inspector-panel">
      <header>
        <h3>Inspector</h3>
        <span>{selectedNode ? selectedNode.kind : "No node"}</span>
      </header>

      {selectedNode ? (
        <>
          <div className="ui-inspector-section">
            <h4>Identity</h4>
            <dl>
              <dt>ID</dt>
              <dd>{selectedNode.id}</dd>
              <dt>Label</dt>
              <dd>{selectedNode.label}</dd>
              <dt>Path</dt>
              <dd title={selectedNode.path}>{selectedNode.path}</dd>
              <dt>Type</dt>
              <dd>{selectedNode.kind}</dd>
            </dl>
          </div>

          <div className="ui-inspector-section">
            <h4>Content</h4>
            <dl>
              <dt>Text</dt>
              <dd>{selectedNode.text ?? "none"}</dd>
              <dt>Style class</dt>
              <dd>{selectedNode.styleClass ?? "none"}</dd>
              <dt>Action</dt>
              <dd>{selectedNode.actionEvent ?? "none"}</dd>
            </dl>
          </div>

          <div className="ui-inspector-actions">
            <button className="button button-ghost" type="button" onClick={onAddChild}>
              <Plus size={14} />
              Add Child
            </button>
            <button className="button button-ghost" type="button" onClick={onDuplicate}>
              <Copy size={14} />
              Duplicate
            </button>
            <button className="button button-ghost" type="button" onClick={onMoveUp}>
              <MoveUp size={14} />
              Move Up
            </button>
            <button className="button button-ghost" type="button" onClick={onMoveDown}>
              <MoveDown size={14} />
              Move Down
            </button>
            <button
              className="button button-danger"
              type="button"
              onClick={onRemove}
              disabled={selectedNode.path === document.root.path}
            >
              <Trash2 size={14} />
              Remove
            </button>
          </div>

          <p className="muted workspace-note">
            Full content/style editing is available in the global Inspector. Structure commands are placeholders in this MVP.
          </p>
        </>
      ) : (
        <p className="muted workspace-note">Select a UI node to inspect it.</p>
      )}
    </section>
  );
}
