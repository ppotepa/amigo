import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { ProjectFileSelection } from "../propertiesTypes";

export function ProjectFilePropertiesPanel({ selection }: { selection: ProjectFileSelection }) {
  const file = selection.file;
  return (
    <KeyValueSection
      title="File"
      rows={[
        { label: "Name", value: file.name },
        { label: "Kind", value: file.kind },
        { label: "Size", value: formatBytes(file.sizeBytes) },
        { label: "Path", value: file.path, title: file.path },
      ]}
    />
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
