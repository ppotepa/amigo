import { LayoutPanelTop } from "lucide-react";
import type { EditorUiDocumentDto } from "../../api/dto";

export function UiDocumentChooserPanel({
  documents,
  onCreateDocument,
  onSelectDocument,
}: {
  documents: EditorUiDocumentDto[];
  onCreateDocument: () => void;
  onSelectDocument: (document: EditorUiDocumentDto) => void;
}) {
  return (
    <section className="ui-document-start">
      <div className="ui-document-start-card ui-document-chooser-card">
        <LayoutPanelTop size={42} />
        <h2>Select UI Document</h2>
        <p>This scene contains multiple UiDocument components. Choose one to edit.</p>

        <div className="ui-document-choice-list">
          {documents.map((document) => (
            <button
              key={`${document.entityId}:${document.componentIndex}`}
              className="ui-document-choice-card"
              type="button"
              onClick={() => onSelectDocument(document)}
            >
              <strong>{document.entityName}</strong>
              <span>UiDocument #{document.componentIndex}</span>
              <em>{document.targetLayer ?? "screen-space"}</em>
            </button>
          ))}
        </div>

        <button className="button button-ghost" type="button" onClick={onCreateDocument}>
          Create another UI Document
        </button>
      </div>
    </section>
  );
}
