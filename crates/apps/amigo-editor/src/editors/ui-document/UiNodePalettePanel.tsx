import { LayoutPanelTop, MousePointerClick } from "lucide-react";
import type { EditorUiNodeDto } from "../../api/dto";
import type { UiNodeCreateKind, UiNodePaletteCategory } from "./uiDocumentEditorTypes";
import { allowedUiChildrenForNode, uiNodeCannotHaveChildrenReason } from "./uiNodeCapabilities";
import { UI_NODE_PALETTE } from "./uiDocumentTemplates";

const CATEGORY_LABELS: Record<UiNodePaletteCategory, string> = {
  layout: "Layout",
  display: "Display",
  controls: "Controls",
  feedback: "Feedback",
};

// @codemap anchor:ui-node-palette-filtered-by-parent domain:ui-document role:palette priority:P1 layer:app tags:add-node,capabilities
export function UiNodePalettePanel({
  parentNode,
  onAddNode,
}: {
  parentNode: EditorUiNodeDto | null;
  onAddNode: (kind: UiNodeCreateKind) => void;
}) {
  const allowedKinds = new Set(allowedUiChildrenForNode(parentNode));
  const categories: UiNodePaletteCategory[] = ["layout", "display", "controls", "feedback"];
  const hasAllowedKinds = allowedKinds.size > 0;

  return (
    <section className="ui-document-panel ui-node-palette-panel">
      <header>
        <h3>Palette</h3>
        <span>{parentNode ? `Parent: ${parentNode.label}` : "Select parent"}</span>
      </header>

      {!hasAllowedKinds ? (
        <p className="ui-node-palette-empty">{uiNodeCannotHaveChildrenReason(parentNode)}</p>
      ) : null}

      {categories.map((category) => {
        const items = UI_NODE_PALETTE.filter(
          (item) => item.category === category && allowedKinds.has(item.kind),
        );

        if (!items.length) return null;

        return (
          <div key={category} className="ui-node-palette-category">
            <h4>{CATEGORY_LABELS[category]}</h4>
            {items.map((item) => (
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
        );
      })}
    </section>
  );
}
