import type { ModSelection } from "../propertiesTypes";

export function ModPropertiesPanel({ selection }: { selection: ModSelection }) {
  const details = selection.details;
  return (
    <>
      <section className="workspace-section">
        <h3>Mod</h3>
        <dl className="kv-list">
          <dt>ID</dt>
          <dd>{details?.id ?? "none"}</dd>
          <dt>Name</dt>
          <dd>{details?.name ?? "none"}</dd>
          <dt>Authors</dt>
          <dd>{details?.authors.join(", ") || "none"}</dd>
          <dt>Root</dt>
          <dd title={details?.rootPath}>{details?.rootPath ?? "none"}</dd>
        </dl>
      </section>
      <section className="workspace-section">
        <h3>Capabilities</h3>
        <div className="tag-list">
          {details?.capabilities.length ? details.capabilities.map((capability) => <span key={capability} className="tag">{capability}</span>) : <span className="muted">No capabilities.</span>}
        </div>
      </section>
    </>
  );
}
