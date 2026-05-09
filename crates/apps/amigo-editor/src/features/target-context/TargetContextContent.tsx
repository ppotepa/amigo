import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import {
  TargetActionsPanel,
  TargetAssetSummaryPanel,
  TargetDiagnosticSummaryPanel,
  TargetDiagnosticsPanel,
  TargetEntityComponentsNavigatorPanel,
  TargetFileSummaryPanel,
  TargetHeaderPanel,
  TargetHistoryPanel,
  TargetProjectSummaryPanel,
  TargetQuickActionsPanel,
  TargetSourcePreviewPanel,
  TargetTraitDetailSectionsPanel,
  TargetTraitSummarySectionsPanel,
} from "./TargetContextSections";

export function TargetContextContent({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const target = services.currentEditorTarget ?? null;

  if (!target) {
    return (
      <div className="dock-scroll target-context-panel">
        <p className="muted workspace-empty">Select an item to inspect its context.</p>
      </div>
    );
  }

  const sectionProps = { services, target };

  return (
    <div className="dock-scroll target-context-panel">
      <TargetHeaderPanel {...sectionProps} />
      {target.ref.kind === "mod" || target.ref.kind === "projectNode" || target.ref.kind === "capability" || target.ref.kind === "dependency" ? (
        <TargetProjectSummaryPanel {...sectionProps} />
      ) : null}
      {target.ref.kind === "projectFile" || target.ref.kind === "script" ? (
        <TargetFileSummaryPanel {...sectionProps} />
      ) : null}
      {target.ref.kind === "asset" ? <TargetAssetSummaryPanel {...sectionProps} /> : null}
      {target.ref.kind === "diagnostic" ? <TargetDiagnosticSummaryPanel {...sectionProps} /> : null}
      {target.ref.kind === "scene" ||
      target.ref.kind === "sceneEntity" ||
      target.ref.kind === "component" ||
      target.ref.kind === "uiDocument" ||
      target.ref.kind === "uiNode" ? (
        <>
          <TargetTraitSummarySectionsPanel {...sectionProps} />
          <TargetEntityComponentsNavigatorPanel {...sectionProps} />
          <TargetTraitDetailSectionsPanel {...sectionProps} />
        </>
      ) : null}
      <TargetQuickActionsPanel {...sectionProps} />
      <TargetActionsPanel {...sectionProps} />
      <TargetSourcePreviewPanel {...sectionProps} />
      <TargetDiagnosticsPanel {...sectionProps} />
      {target.ref.kind === "scene" || target.ref.kind === "uiNode" ? (
        <TargetHistoryPanel {...sectionProps} />
      ) : null}
    </div>
  );
}
