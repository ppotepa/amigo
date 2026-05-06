import { ShowYamlButton } from "../../features/files/ShowYamlButton";
import { projectFileYamlSource } from "../../features/files/yamlSourceRefs";
import { KeyValueSection } from "../../ui/properties/KeyValueSection";
import type { ProjectFileSelection, PropertiesContext } from "../propertiesTypes";

export function ProjectFilePropertiesPanel({
  context,
  selection,
}: {
  context: PropertiesContext;
  selection: ProjectFileSelection;
}) {
  const file = selection.file;
  const source = projectFileYamlSource(file);
  return (
    <>
      <KeyValueSection
        title="File"
        rows={[
          { label: "Name", value: file.name },
          { label: "Kind", value: file.kind },
          { label: "Size", value: formatBytes(file.sizeBytes) },
          { label: "Path", value: file.path, title: file.path },
        ]}
      />

      {source ? (
        <section className="workspace-section">
          <h3>Source</h3>
          <ShowYamlButton source={source} onShow={context.onShowYaml} />
        </section>
      ) : null}
    </>
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
