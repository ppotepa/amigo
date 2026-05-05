import type { EntitySelection } from "../propertiesTypes";

export function EntityPropertiesPanel({ selection }: { selection: EntitySelection }) {
  const entity = selection.entity;
  return (
    <section className="workspace-section">
      <h3>Entity</h3>
      <dl className="kv-list">
        <dt>Name</dt>
        <dd>{entity.name}</dd>
        <dt>Scene</dt>
        <dd>{selection.scene?.label ?? selection.scene?.id ?? "none"}</dd>
        <dt>Visible</dt>
        <dd>{entity.visible ? "yes" : "no"}</dd>
        <dt>Simulation</dt>
        <dd>{entity.simulationEnabled ? "enabled" : "disabled"}</dd>
        <dt>Collision</dt>
        <dd>{entity.collisionEnabled ? "enabled" : "disabled"}</dd>
        <dt>Transforms</dt>
        <dd>{[entity.hasTransform2 ? "2D" : null, entity.hasTransform3 ? "3D" : null].filter(Boolean).join(", ") || "none"}</dd>
        <dt>Properties</dt>
        <dd>{entity.propertyCount}</dd>
      </dl>
      <div className="tag-list workspace-component-tags">
        {entity.componentTypes.length ? (
          entity.componentTypes.map((component, index) => <span key={`${component}:${index}`} className="tag">{component}</span>)
        ) : (
          <span className="muted">No components.</span>
        )}
      </div>
    </section>
  );
}
