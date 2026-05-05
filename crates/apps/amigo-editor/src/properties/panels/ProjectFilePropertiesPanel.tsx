import type { ProjectFileSelection } from "../propertiesTypes";

export function ProjectFilePropertiesPanel({ selection }: { selection: ProjectFileSelection }) {
  const file = selection.file;
  return (
    <section className="workspace-section">
      <h3>File</h3>
      <dl className="kv-list">
        <dt>Name</dt>
        <dd>{file.name}</dd>
        <dt>Kind</dt>
        <dd>{file.kind}</dd>
        <dt>Size</dt>
        <dd>{formatBytes(file.sizeBytes)}</dd>
        <dt>Path</dt>
        <dd title={file.path}>{file.path}</dd>
      </dl>
    </section>
  );
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}
