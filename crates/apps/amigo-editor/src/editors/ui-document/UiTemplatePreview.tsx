import {
  HeartPulse,
  LayoutPanelTop,
  MessageSquare,
  MonitorCog,
  MousePointerClick,
  Rows3,
} from "lucide-react";
import type { UiTemplateDefinition } from "./uiDocumentEditorTypes";

export function UiTemplatePreview({
  template,
}: {
  template: UiTemplateDefinition | null;
}) {
  if (!template) {
    return <div className="ui-template-preview-empty">Select a template.</div>;
  }

  return (
    <aside className="ui-template-preview ui-template-preview-large">
      <div className="ui-template-preview-visual">
        {iconForTemplate(template.kind)}
        <strong>{template.previewLabel}</strong>
      </div>

      <h3>{template.label}</h3>
      <p>{template.description}</p>

      <div className="ui-template-preview-details">
        <strong>Creates</strong>
        <ul>
          {detailsForTemplate(template.kind).map((detail) => (
            <li key={detail}>{detail}</li>
          ))}
        </ul>
      </div>

      {!template.enabled ? (
        <p className="ui-dialog-error">{template.disabledReason ?? "Coming soon"}</p>
      ) : null}
    </aside>
  );
}

function iconForTemplate(kind: string) {
  switch (kind) {
    case "vertical-menu":
      return <MousePointerClick size={44} />;
    case "button-row":
      return <Rows3 size={44} />;
    case "health-bar":
      return <HeartPulse size={44} />;
    case "dialogue-box":
      return <MessageSquare size={44} />;
    case "options-group":
      return <MonitorCog size={44} />;
    case "empty-document":
    default:
      return <LayoutPanelTop size={44} />;
  }
}

function detailsForTemplate(kind: string): string[] {
  switch (kind) {
    case "vertical-menu":
      return ["root container", "menu panel", "Start / Options / Quit buttons"];
    case "button-row":
      return ["row container", "three button placeholders"];
    case "health-bar":
      return ["HUD row", "label", "progress bar placeholder"];
    case "ammo-counter":
      return ["HUD label", "ammo value text"];
    case "dialogue-box":
      return ["dialogue panel", "speaker text", "body text", "continue hint"];
    case "options-group":
      return ["options container", "setting rows", "placeholder controls"];
    case "empty-document":
    default:
      return ["root container only"];
  }
}
