import { HeartPulse, LayoutPanelTop, MessageSquare, MousePointerClick } from "lucide-react";
import type { UiTemplateKind } from "./uiDocumentEditorTypes";

export function UiDocumentStartScreen({
  mode = "emptyScene",
  message,
  preferredEntityId,
  onCreateDocument,
}: {
  mode?: "emptyScene" | "missingTarget" | "noScene";
  message?: string;
  preferredEntityId?: string | null;
  onCreateDocument: (template?: UiTemplateKind) => void;
}) {
  const disabled = mode === "noScene";

  return (
    <section className="ui-document-start">
      <div className="ui-document-start-card">
        <LayoutPanelTop size={42} />
        <h2>
          {mode === "noScene"
            ? "Select a scene"
            : mode === "missingTarget"
              ? "Create missing UI Document"
              : "Create UI Document"}
        </h2>

        <p>
          {message ??
            (mode === "noScene"
              ? "Choose a scene before creating a UI document."
              : "This scene has no UiDocument yet. Create one to start editing UI.")}
        </p>

        {preferredEntityId ? (
          <p className="dialog-muted-note">
            Suggested entity: <code>{preferredEntityId}</code>
          </p>
        ) : null}

        <div className="ui-template-quick-grid">
          <button
            className="ui-template-card"
            type="button"
            disabled={disabled}
            onClick={() => onCreateDocument("empty-document")}
          >
            <span className="ui-template-card-icon">
              <LayoutPanelTop size={24} />
            </span>
            <strong>Blank UI</strong>
            <span>Minimal root container.</span>
          </button>

          <button
            className="ui-template-card"
            type="button"
            disabled={disabled}
            onClick={() => onCreateDocument("vertical-menu")}
          >
            <span className="ui-template-card-icon">
              <MousePointerClick size={24} />
            </span>
            <strong>Main Menu</strong>
            <span>Title and vertical button list.</span>
          </button>

          <button
            className="ui-template-card"
            type="button"
            disabled={disabled}
            onClick={() => onCreateDocument("health-bar")}
          >
            <span className="ui-template-card-icon">
              <HeartPulse size={24} />
            </span>
            <strong>HUD Overlay</strong>
            <span>Simple status/progress UI.</span>
          </button>

          <button
            className="ui-template-card"
            type="button"
            disabled={disabled}
            onClick={() => onCreateDocument("dialogue-box")}
          >
            <span className="ui-template-card-icon">
              <MessageSquare size={24} />
            </span>
            <strong>Dialogue Box</strong>
            <span>Speaker, text and continue hint.</span>
          </button>
        </div>
      </div>
    </section>
  );
}
