import { Copy, MoveDown, MoveUp, Plus, Trash2 } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";

// @codemap anchor:ui-node-actions-panel domain:ui-document role:inspector priority:P1 layer:app tags:ui-document,node-actions,structure-dock
export function UiNodeActionsPanel({
  busy,
  canAddChild,
  canDuplicate,
  canMoveDown,
  canMoveUp,
  canRemove,
  document,
  selectedNode,
  onAddChild,
  onDuplicate,
  onMoveDown,
  onMoveUp,
  onRemove,
}: {
  busy: boolean;
  canAddChild: boolean;
  canDuplicate: boolean;
  canMoveDown: boolean;
  canMoveUp: boolean;
  canRemove: boolean;
  document: EditorUiDocumentDto;
  selectedNode: EditorUiNodeDto | null;
  onAddChild: () => void;
  onDuplicate: () => void;
  onMoveDown: () => void;
  onMoveUp: () => void;
  onRemove: () => void;
}) {
  return (
    <section className="ui-document-panel ui-node-actions-panel">
      <header>
        <h3>Node Actions</h3>
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
            <button className="button button-ghost" type="button" disabled={busy || !canAddChild} onClick={onAddChild}>
              <Plus size={14} />
              Add Child
            </button>
            <button className="button button-ghost" type="button" disabled={busy || !canDuplicate} onClick={onDuplicate}>
              <Copy size={14} />
              Duplicate
            </button>
            <button className="button button-ghost" type="button" disabled={busy || !canMoveUp} onClick={onMoveUp}>
              <MoveUp size={14} />
              Move Up
            </button>
            <button className="button button-ghost" type="button" disabled={busy || !canMoveDown} onClick={onMoveDown}>
              <MoveDown size={14} />
              Move Down
            </button>
            <button
              className="button button-danger"
              type="button"
              onClick={onRemove}
              disabled={busy || !canRemove || selectedNode.path === document.root.path}
            >
              <Trash2 size={14} />
              Remove
            </button>
          </div>

          <p className="muted workspace-note">
            Full content/style editing is available in the global Inspector.
          </p>
        </>
      ) : (
        <p className="muted workspace-note">Select a UI node to inspect it.</p>
      )}
    </section>
  );
}
