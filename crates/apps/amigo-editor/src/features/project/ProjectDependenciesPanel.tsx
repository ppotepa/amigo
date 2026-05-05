import { Package } from "lucide-react";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function ProjectDependenciesPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const details = services.details ?? null;
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Package size={14} /></span>
        <div>
          <h2>Dependencies</h2>
          <p>{details?.id ?? "No mod"}</p>
        </div>
      </header>
      <section className="workspace-section">
        <h3>Dependencies</h3>
        {details?.dependencies.length ? details.dependencies.map((dependency) => {
          const missing = details.missingDependencies.includes(dependency);
          return (
            <div key={dependency} className="workspace-row">
              <span className="dock-icon dock-icon-blue"><Package size={13} /></span>
              <span>
                <strong>{dependency}</strong>
                <small>{missing ? "missing from mods root" : "available"}</small>
              </span>
              <em className={`badge ${missing ? "badge-error" : "badge-valid"}`}>{missing ? "missing" : "ok"}</em>
            </div>
          );
        }) : <p className="muted">No dependencies.</p>}
      </section>
    </div>
  );
}
