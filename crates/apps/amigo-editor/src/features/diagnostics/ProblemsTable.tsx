import { AlertTriangle } from "lucide-react";
import type { EditorDiagnosticDto } from "../../api/dto";
import { diagnosticToTarget } from "../../editor-targets/adapters/diagnosticTargetAdapter";
import { editorTargetKey } from "../../editor-targets/editorTargetTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { semanticIconClass, toneForStatus } from "../../theme/semanticColorRegistry";

// @codemap anchor:problems-table-target-wiring domain:workspace role:tree priority:P1 layer:app tags:diagnostics,editor-target
export function ProblemsTable({
  services,
}: {
  services: WorkspaceRuntimeServices;
}) {
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
          <ProblemRow
            key={`${diagnostic.code}:${index}`}
            diagnostic={diagnostic}
            index={index}
            services={services}
          />
        ))}
      </tbody>
    </table>
  );
}

function ProblemRow({
  diagnostic,
  index,
  services,
}: {
  diagnostic: EditorDiagnosticDto;
  index: number;
  services: WorkspaceRuntimeServices;
}) {
  const target = diagnosticToTarget(diagnostic, index);
  const selected = services.currentEditorTarget
    ? editorTargetKey(services.currentEditorTarget.ref) === editorTargetKey(target)
    : false;

  return (
    <tr
      className={selected ? "selected" : ""}
      onClick={() => services.activateEditorTarget?.(target, "select")}
      onDoubleClick={() => services.activateEditorTarget?.(target, "open")}
      onContextMenu={(event) => {
        event.preventDefault();
        services.activateEditorTarget?.(target, "contextMenu");
      }}
    >
      <td>
        <span className={`badge diagnostic-${diagnostic.level}`}>
          <AlertTriangle size={12} className={semanticIconClass(toneForStatus(diagnostic.level))} />
          {diagnostic.level}
        </span>
      </td>
      <td>{diagnostic.code}</td>
      <td>{diagnostic.message}</td>
      <td>{diagnostic.path ?? ""}</td>
    </tr>
  );
}
