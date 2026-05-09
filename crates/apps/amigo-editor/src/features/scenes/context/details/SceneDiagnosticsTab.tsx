import { AlertTriangle } from "lucide-react";
import { ContextActionStrip } from "../../../../ui/context-dock/ContextActionStrip";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import type { WorkspaceRuntimeServices } from "../../../../main-window/workspaceRuntimeServices";
import { sceneContextIcon } from "../sceneContextIcons";
import type { SceneContextModel } from "../sceneContextTypes";

export function SceneDiagnosticsTab({
  model,
  services,
}: {
  model: SceneContextModel;
  services: WorkspaceRuntimeServices;
}) {
  const diagnostics = services.sceneValidation?.diagnostics?.length
    ? services.sceneValidation.diagnostics
    : model.diagnosticsInfo.diagnostics;
  const errorCount = diagnostics.filter((diagnostic) => diagnostic.level === "error").length;
  const warningCount = diagnostics.filter((diagnostic) => diagnostic.level === "warning").length;

  return (
    <div className="scene-detail-tab">
      <section className="scene-detail-section">
        <ContextActionStrip
          actions={[{
            id: "validate-scene",
            label: "Validate Scene",
            icon: <AlertTriangle size={13} />,
            disabled: !services.requestValidateScene,
            onClick: () => void services.requestValidateScene?.(model.scene.id),
          }]}
        />
      </section>

      <section className="scene-detail-section">
        <header className="scene-detail-section-header">
          <span>{diagnostics.length} diagnostics</span>
          <small>{errorCount} errors, {warningCount} warnings</small>
        </header>
        {diagnostics.length ? diagnostics.map((diagnostic, index) => (
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
      </section>
    </div>
  );
}
