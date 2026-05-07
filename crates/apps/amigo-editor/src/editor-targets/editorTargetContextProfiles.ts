import type { EditorTargetKind } from "./editorTargetTypes";
import {
  normalizeEditorTargetContextProfile,
  type EditorTargetContextProfile,
  type EditorTargetContextProfileRegistry,
} from "./editorTargetContextTypes";
import {
  TargetActionsPanel,
  TargetAssetSummaryPanel,
  TargetDiagnosticSummaryPanel,
  TargetDiagnosticsPanel,
  TargetEntitySummaryPanel,
  TargetFileSummaryPanel,
  TargetHeaderPanel,
  TargetHistoryPanel,
  TargetProjectSummaryPanel,
  TargetQuickActionsPanel,
  TargetSceneSummaryPanel,
  TargetSourcePreviewPanel,
  TargetUiSummaryPanel,
} from "../features/target-context/TargetContextSections";

// @codemap anchor:editor-target-context-profiles domain:workspace role:model priority:P1 layer:app tags:editor-target,right-dock,profile,item-context
export const EDITOR_TARGET_CONTEXT_PROFILES = {
  mod: {
    primary: [TargetHeaderPanel, TargetProjectSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  projectNode: {
    primary: [TargetHeaderPanel, TargetProjectSummaryPanel, TargetQuickActionsPanel],
    secondary: TargetActionsPanel,
    defaultAction: "open",
  },
  projectFile: {
    primary: [TargetHeaderPanel, TargetFileSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetSourcePreviewPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  script: {
    primary: [TargetHeaderPanel, TargetFileSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetSourcePreviewPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  asset: {
    primary: [TargetHeaderPanel, TargetAssetSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetSourcePreviewPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  scene: {
    primary: [TargetHeaderPanel, TargetSceneSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetDiagnosticsPanel, TargetHistoryPanel],
    defaultAction: "open",
  },
  sceneEntity: {
    primary: [TargetHeaderPanel, TargetEntitySummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetDiagnosticsPanel],
    defaultAction: "focusViewport",
  },
  uiDocument: {
    primary: [TargetHeaderPanel, TargetUiSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  uiNode: {
    primary: [TargetHeaderPanel, TargetUiSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetHistoryPanel, TargetDiagnosticsPanel],
    defaultAction: "open",
  },
  diagnostic: {
    primary: [TargetHeaderPanel, TargetDiagnosticSummaryPanel, TargetQuickActionsPanel],
    secondary: [TargetActionsPanel, TargetSourcePreviewPanel],
    defaultAction: "reveal",
  },
  capability: {
    primary: [TargetHeaderPanel, TargetProjectSummaryPanel, TargetQuickActionsPanel],
    secondary: TargetActionsPanel,
    defaultAction: "open",
  },
  dependency: {
    primary: [TargetHeaderPanel, TargetProjectSummaryPanel, TargetQuickActionsPanel],
    secondary: TargetActionsPanel,
    defaultAction: "open",
  },
} satisfies EditorTargetContextProfileRegistry;

export function editorTargetContextProfileFor(
  kind: EditorTargetKind,
): EditorTargetContextProfile {
  return normalizeEditorTargetContextProfile(EDITOR_TARGET_CONTEXT_PROFILES[kind]);
}
