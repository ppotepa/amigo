import { AlertCircle, AlertTriangle, Info } from "lucide-react";
import type { EditorDiagnosticDto } from "../api/dto";

export function DiagnosticsList({ diagnostics }: { diagnostics: EditorDiagnosticDto[] }) {
  if (diagnostics.length === 0) {
    return <p className="muted">No diagnostics.</p>;
  }

  return (
    <div className="diagnostics-list">
      {diagnostics.map((diagnostic, index) => (
        <div key={`${diagnostic.code}:${index}`} className={`diagnostic diagnostic-${diagnostic.level}`}>
          {diagnostic.level === "error" ? <AlertCircle size={15} /> : diagnostic.level === "warning" ? <AlertTriangle size={15} /> : <Info size={15} />}
          <div>
            <strong>{diagnostic.code}</strong>
            <p>{diagnostic.message}</p>
            {diagnostic.path ? <small>{diagnostic.path}</small> : null}
          </div>
        </div>
      ))}
    </div>
  );
}
