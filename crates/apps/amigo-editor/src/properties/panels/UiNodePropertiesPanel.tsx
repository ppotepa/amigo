import { MousePointer2 } from "lucide-react";
import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { UiNodeSelection } from "../propertiesTypes";

export function UiNodePropertiesPanel({ selection }: { selection: UiNodeSelection }) {
  const { node, entity, nodeRef, scene } = selection;

  return (
    <>
      <KeyValueSection
        title="UI Node"
        rows={[
          { label: "ID", value: node.id },
          { label: "Label", value: node.label },
          { label: "Type", value: node.kind },
          { label: "Path", value: node.path, title: node.path },
          { label: "Scene", value: scene?.label ?? "none", title: scene?.id },
          { label: "Entity", value: entity.name, title: entity.id },
          { label: "Component", value: `UiDocument #${nodeRef.componentIndex}` },
        ]}
      />

      <KeyValueSection
        title="Content"
        rows={[
          { label: "Text", value: node.text ?? "none" },
          { label: "Style class", value: node.styleClass ?? "none" },
          { label: "Visible", value: node.visible ? "yes" : "no" },
          { label: "Enabled", value: node.enabled ? "yes" : "no" },
          { label: "Children", value: String(node.childCount) },
        ]}
      />

      <KeyValueSection
        title="Action"
        rows={[
          { label: "on_click", value: node.actionEvent ?? "none", title: node.actionEvent ?? undefined },
        ]}
      />

      <section className="workspace-section">
        <h3>Editor</h3>
        <div className="workspace-row">
          <span className="dock-icon dock-icon-blue">
            <MousePointer2 size={14} />
          </span>
          <span>
            <strong>Canvas selection</strong>
            <small>This UI node can be selected on the editor canvas. Editing comes in the next step.</small>
          </span>
          <em className="badge badge-muted">read-only</em>
        </div>
      </section>
    </>
  );
}
