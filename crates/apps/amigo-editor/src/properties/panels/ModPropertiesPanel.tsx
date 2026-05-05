import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { ModSelection } from "../propertiesTypes";

export function ModPropertiesPanel({ selection }: { selection: ModSelection }) {
  const details = selection.details;
  return (
    <>
      <KeyValueSection
        title="Mod"
        rows={[
          { label: "ID", value: details?.id ?? "none" },
          { label: "Name", value: details?.name ?? "none" },
          { label: "Authors", value: details?.authors.join(", ") || "none" },
          { label: "Root", value: details?.rootPath ?? "none", title: details?.rootPath },
        ]}
      />
      <section className="workspace-section">
        <h3>Capabilities</h3>
        <div className="tag-list">
          {details?.capabilities.length ? details.capabilities.map((capability) => <span key={capability} className="tag">{capability}</span>) : <span className="muted">No capabilities.</span>}
        </div>
      </section>
    </>
  );
}
