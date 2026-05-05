import { Boxes } from "lucide-react";
import type { EditorModDetailsDto } from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function ProjectOverviewPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return <ProjectOverview details={services.details ?? null} />;
}

function ProjectOverview({ details }: { details: EditorModDetailsDto | null }) {
  if (!details) {
    return <p className="muted workspace-empty">No mod loaded.</p>;
  }
  return (
    <div className="project-center-panel">
      <header>
        <span className="dock-icon dock-icon-blue"><Boxes size={14} /></span>
        <div>
          <h2>{details.name}</h2>
          <p>{details.id} · {details.version}</p>
        </div>
        <span className={`badge status-${details.status}`}>{details.status}</span>
      </header>
      <section className="workspace-section">
        <h3>Summary</h3>
        <dl className="kv-list">
          <dt>Root</dt>
          <dd title={details.rootPath}>{details.rootPath}</dd>
          <dt>Authors</dt>
          <dd>{details.authors.join(", ") || "none"}</dd>
          <dt>Scenes</dt>
          <dd>{details.sceneCount} total · {details.visibleSceneCount} launcher visible</dd>
          <dt>Files</dt>
          <dd>{details.contentSummary.totalFiles}</dd>
          <dt>Diagnostics</dt>
          <dd>{details.diagnostics.length}</dd>
        </dl>
      </section>
      <section className="workspace-section">
        <h3>Description</h3>
        <p className="muted">{details.description || "No description."}</p>
      </section>
      <ContentBreakdown details={details} />
    </div>
  );
}

function ContentBreakdown({ details }: { details: EditorModDetailsDto }) {
  const rows = [
    ["Scene YAML", details.contentSummary.sceneYaml],
    ["Scripts", details.contentSummary.scripts],
    ["Textures", details.contentSummary.textures],
    ["Spritesheets", details.contentSummary.spritesheets],
    ["Tilemaps", details.contentSummary.tilemaps],
    ["Tilesets", details.contentSummary.tilesets],
    ["Packages", details.contentSummary.packages],
    ["Unknown", details.contentSummary.unknownFiles],
  ] as const;
  return (
    <section className="workspace-section">
      <h3>Content Breakdown</h3>
      <div className="workspace-count-list two-column">
        {rows.map(([label, value]) => <CountRow key={label} label={label} value={value} />)}
      </div>
    </section>
  );
}

function CountRow({ label, value }: { label: string; value: number }) {
  return (
    <div className="workspace-count-row">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
