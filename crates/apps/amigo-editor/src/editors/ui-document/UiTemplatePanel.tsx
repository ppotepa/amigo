import { Boxes } from "lucide-react";
import type { UiTemplateKind } from "./uiDocumentEditorTypes";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function UiTemplatePanel({ onAddTemplate }: { onAddTemplate: (template: UiTemplateKind) => void }) {
  return (
    <section className="ui-document-panel ui-template-panel">
      <header>
        <h3>Templates</h3>
        <span>Add Template</span>
      </header>

      <div className="ui-template-grid">
        {UI_TEMPLATE_DEFINITIONS.filter((template) => template.category !== "document").map((template) => (
          <button
            key={template.kind}
            className="ui-template-card"
            disabled={!template.enabled}
            type="button"
            onClick={() => onAddTemplate(template.kind)}
          >
            <span className="dock-icon dock-icon-purple">
              <Boxes size={15} />
            </span>
            <strong>{template.label}</strong>
            <small>{template.description}</small>
            {!template.enabled ? <em>{template.disabledReason ?? "Coming soon"}</em> : null}
          </button>
        ))}
      </div>
    </section>
  );
}
