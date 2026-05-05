import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { EntitySelection } from "../propertiesTypes";

export function EntityPropertiesPanel({ selection }: { selection: EntitySelection }) {
  const entity = selection.entity;
  return (
    <>
      <KeyValueSection
        title="Entity"
        rows={[
          { label: "Name", value: entity.name },
          { label: "Scene", value: selection.scene?.label ?? selection.scene?.id ?? "none" },
          { label: "Visible", value: entity.visible ? "yes" : "no" },
          { label: "Simulation", value: entity.simulationEnabled ? "enabled" : "disabled" },
          { label: "Collision", value: entity.collisionEnabled ? "enabled" : "disabled" },
          {
            label: "Transforms",
            value:
              [entity.hasTransform2 ? "2D" : null, entity.hasTransform3 ? "3D" : null]
                .filter(Boolean)
                .join(", ") || "none",
          },
          { label: "Properties", value: entity.propertyCount },
        ]}
      />
      <section className="workspace-section">
        <h3>Components</h3>
      <div className="tag-list workspace-component-tags">
        {entity.componentTypes.length ? (
          entity.componentTypes.map((component, index) => <span key={`${component}:${index}`} className="tag">{component}</span>)
        ) : (
          <span className="muted">No components.</span>
        )}
      </div>
      </section>
    </>
  );
}
