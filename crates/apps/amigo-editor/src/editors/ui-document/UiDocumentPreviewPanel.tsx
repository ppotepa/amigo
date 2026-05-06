import { Monitor, PanelTop } from "lucide-react";
import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";

export function UiDocumentPreviewPanel({
  document,
  selectedNode,
}: {
  document: EditorUiDocumentDto;
  selectedNode: EditorUiNodeDto | null;
}) {
  return (
    <section className="ui-document-preview-panel">
      <header>
        <div>
          <h3>Preview</h3>
          <span>{document.targetLayer ?? "screen-space"}</span>
        </div>
        <div className="ui-preview-toolbar">
          <button className="button button-ghost" type="button">
            <Monitor size={14} />
            100%
          </button>
        </div>
      </header>

      <div className="ui-preview-canvas">
        <div className="ui-preview-artboard">
          <PanelTop size={44} />
          <strong>{document.entityName}</strong>
          <span>Preview is structure-first in MVP.</span>

          {selectedNode ? (
            <div className="ui-preview-selection">
              Selected: <code>{selectedNode.path}</code>
            </div>
          ) : (
            <div className="ui-preview-selection muted">Select a node in the tree or scene canvas.</div>
          )}
        </div>
      </div>
    </section>
  );
}
