import { FilePlus2, LayoutPanelTop } from "lucide-react";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function UiDocumentStartScreen({ onCreateDocument }: { onCreateDocument: () => void }) {
  const documentTemplates = UI_TEMPLATE_DEFINITIONS.filter(
    (template) =>
      template.category === "document" ||
      template.category === "menu" ||
      template.category === "hud" ||
      template.category === "dialogue",
  ).slice(0, 5);

  return (
    <section className="ui-document-start">
      <div className="ui-document-start-card">
        <LayoutPanelTop size={42} />
        <h2>UI Document Editor</h2>
        <p>Create or open a UiDocument to start building interface screens, HUDs and menus.</p>

        <button className="button button-primary" type="button" onClick={onCreateDocument}>
          <FilePlus2 size={16} />
          Create UI Document
        </button>

        <div className="ui-template-quick-grid">
          {documentTemplates.map((template) => (
            <button
              key={template.kind}
              className="ui-template-card"
              disabled={!template.enabled}
              type="button"
              onClick={onCreateDocument}
            >
              <strong>{template.label}</strong>
              <span>{template.description}</span>
              {!template.enabled ? <em>{template.disabledReason ?? "Coming soon"}</em> : null}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
