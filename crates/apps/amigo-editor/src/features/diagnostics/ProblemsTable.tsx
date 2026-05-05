import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function ProblemsTable({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const diagnostics = services.allProblems ?? [];
  const level = String(services.toolbarState?.level ?? "all");
  const filteredDiagnostics = diagnostics.filter((diagnostic) => {
    if (level === "all") return true;
    return diagnostic.level === level;
  });

  if (filteredDiagnostics.length === 0) {
    return <p className="muted workspace-empty">No problems.</p>;
  }
  return (
    <table className="workspace-table">
      <tbody>
        {filteredDiagnostics.map((diagnostic, index) => (
          <tr key={`${diagnostic.code}:${index}`}>
            <td><span className={`badge diagnostic-${diagnostic.level}`}>{diagnostic.level}</span></td>
            <td>{diagnostic.code}</td>
            <td>{diagnostic.message}</td>
            <td>{diagnostic.path ?? ""}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
