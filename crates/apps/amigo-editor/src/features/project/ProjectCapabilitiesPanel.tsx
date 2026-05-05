import { Plug } from "lucide-react";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function ProjectCapabilitiesPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const details = services.details ?? null;
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Plug size={14} /></span>
        <div>
          <h2>Capabilities</h2>
          <p>{details?.id ?? "No mod"}</p>
        </div>
      </header>
      <section className="workspace-section">
        <h3>Declared Capabilities</h3>
        <div className="project-token-grid">
          {details?.capabilities.length ? details.capabilities.map((capability) => (
            <span key={capability} className="tag">{capability}</span>
          )) : <p className="muted">No capabilities declared.</p>}
        </div>
      </section>
    </div>
  );
}
