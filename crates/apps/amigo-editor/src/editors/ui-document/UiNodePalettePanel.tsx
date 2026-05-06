import { LayoutPanelTop, MousePointerClick } from "lucide-react";
import type { UiNodeCreateKind, UiNodePaletteCategory } from "./uiDocumentEditorTypes";
import { UI_NODE_PALETTE } from "./uiDocumentTemplates";

const CATEGORY_LABELS: Record<UiNodePaletteCategory, string> = {
  layout: "Layout",
  display: "Display",
  controls: "Controls",
  feedback: "Feedback",
};

export function UiNodePalettePanel({ onAddNode }: { onAddNode: (kind: UiNodeCreateKind) => void }) {
  const categories: UiNodePaletteCategory[] = ["layout", "display", "controls", "feedback"];

  return (
    <section className="ui-document-panel ui-node-palette-panel">
      <header>
        <h3>Palette</h3>
        <span>Add Node</span>
      </header>

      {categories.map((category) => (
        <div key={category} className="ui-node-palette-category">
          <h4>{CATEGORY_LABELS[category]}</h4>
          {UI_NODE_PALETTE.filter((item) => item.category === category).map((item) => (
            <button
              key={item.kind}
              className="ui-node-palette-item"
              disabled={!item.enabled}
              type="button"
              onClick={() => onAddNode(item.kind)}
            >
              <span className="dock-icon dock-icon-cyan">
                {item.category === "layout" ? <LayoutPanelTop size={14} /> : <MousePointerClick size={14} />}
              </span>
              <span>
                <strong>{item.label}</strong>
                <small>{item.enabled ? item.description : item.disabledReason ?? item.description}</small>
              </span>
            </button>
          ))}
        </div>
      ))}
    </section>
  );
}
