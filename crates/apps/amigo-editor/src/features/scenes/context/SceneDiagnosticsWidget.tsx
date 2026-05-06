import type { EditorDiagnosticDto } from "../../../api/dto";
import { ContextRow } from "../../../ui/context-dock/ContextRow";
import { ContextWidget } from "../../../ui/context-dock/ContextWidget";
import { sceneContextIcon } from "./sceneContextIcons";

export function SceneDiagnosticsWidget({
  diagnostics,
}: {
  diagnostics: EditorDiagnosticDto[];
}) {
  const hasProblems = diagnostics.length > 0;

  return (
    <ContextWidget
      id="scene-diagnostics"
      title="Diagnostics"
      icon={sceneContextIcon("diagnostics")}
      badge={diagnostics.length}
      badgeTone={hasProblems ? "warning" : "valid"}
      defaultCollapsed={!hasProblems}
    >
      {hasProblems ? diagnostics.map((diagnostic, index) => (
        <ContextRow
          key={`${diagnostic.code}:${diagnostic.path ?? ""}:${index}`}
          icon={sceneContextIcon("diagnostics")}
          title={diagnostic.code}
          subtitle={diagnostic.message}
          badge={diagnostic.level}
          tone={diagnostic.level === "error" ? "red" : "orange"}
        />
      )) : (
        <ContextRow
          icon={sceneContextIcon("diagnostics")}
          title="No problems"
          subtitle="Scene diagnostics are clean."
          badge="OK"
          tone="green"
        />
      )}
    </ContextWidget>
  );
}
