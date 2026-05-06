import {
  FilePlus2,
  HeartPulse,
  LayoutPanelTop,
  MessageSquare,
  MonitorCog,
  MousePointerClick,
} from "lucide-react";
import type { UiTemplateKind } from "./uiDocumentEditorTypes";
import { UI_TEMPLATE_DEFINITIONS } from "./uiDocumentTemplates";

export function UiDocumentStartScreen({
  onCreateBlank,
  onCreateFromTemplate,
}: {
  onCreateBlank: () => void;
  onCreateFromTemplate: (template: UiTemplateKind) => void;
}) {
  const documentTemplates = UI_TEMPLATE_DEFINITIONS.filter(
    (template) =>
      template.kind === "empty-document" ||
      template.kind === "vertical-menu" ||
      template.kind === "health-bar" ||
      template.kind === "dialogue-box" ||
      template.kind === "options-group",
  );

  return (
    <section className="ui-document-start">
      <div className="ui-document-start-card">
        <LayoutPanelTop size={42} />
        <h2>UI Document Editor</h2>
        <p>Create a new screen-space UI document, or choose a template to start faster.</p>

        <button className="button button-primary" type="button" onClick={onCreateBlank}>
          <FilePlus2 size={16} />
          Create Blank UI
        </button>

        <div className="ui-template-quick-grid ui-template-quick-grid-readable">
          {documentTemplates.map((template) => (
            <button
              key={template.kind}
              className="ui-template-card ui-document-template-card"
              disabled={!template.enabled}
              type="button"
              onClick={() => onCreateFromTemplate(template.kind)}
            >
              <span className="ui-template-card-icon">{iconForTemplate(template.kind)}</span>
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

function iconForTemplate(template: UiTemplateKind) {
  switch (template) {
    case "vertical-menu":
      return <MousePointerClick size={24} />;
    case "health-bar":
      return <HeartPulse size={24} />;
    case "dialogue-box":
      return <MessageSquare size={24} />;
    case "options-group":
      return <MonitorCog size={24} />;
    case "empty-document":
    default:
      return <LayoutPanelTop size={24} />;
  }
}
