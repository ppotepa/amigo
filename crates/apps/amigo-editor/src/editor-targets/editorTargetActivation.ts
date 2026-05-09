import {
  DiagnosticsProblemsComponent,
  ProjectCapabilitiesComponent,
  ProjectDependenciesComponent,
  ProjectOverviewComponent,
  UiDocumentEditorComponent,
  editorComponentById,
} from "../editor-components/componentRegistry";
import type { EditorComponentDefinition } from "../editor-components/componentTypes";
import type { OpenWorkspaceEditorRequest } from "../main-window/workspaceOpenTypes";
import type {
  EditorTargetRuntimeBridge,
  WorkspaceRuntimeServices,
} from "../main-window/workspaceRuntimeServices";
import { resolveEditorTarget } from "./editorTargetResolver";
import type {
  EditorTargetIntent,
  EditorTargetRef,
  ResolvedEditorTarget,
} from "./editorTargetTypes";

export type EditorTargetActivationResult = {
  resolved: ResolvedEditorTarget;
  opened: boolean;
  revealed: boolean;
};

// @codemap anchor:editor-target-activation domain:workspace role:dispatcher priority:P1 layer:app tags:editor-target,selection,open-routing
export function activateEditorTarget(
  target: EditorTargetRef,
  intent: EditorTargetIntent,
  services: WorkspaceRuntimeServices,
): EditorTargetActivationResult {
  const resolved = resolveEditorTarget(target, services);

    if (intent === "open") {
      const opened = openResolvedEditorTarget(resolved, services.targetBridge);
      return {
        resolved,
        opened,
        revealed: opened ? false : revealResolvedEditorTarget(resolved, services),
      };
    }

  if (intent === "reveal") {
    return {
      resolved,
      opened: false,
      revealed: revealResolvedEditorTarget(resolved, services),
    };
  }

  if (intent === "contextMenu") {
    services.recordEvent?.({
      type: "EditorTargetContextMenuRequested",
      targetKind: target.kind,
      targetLabel: resolved.descriptor.label,
    });
    return { resolved, opened: false, revealed: false };
  }

  return { resolved, opened: false, revealed: false };
}

// @codemap anchor:editor-target-open-routing domain:workspace role:dispatcher priority:P1 layer:app tags:editor-target,open-routing
export function openResolvedEditorTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.status !== "resolved") return false;
  const strategy = OPEN_TARGET_STRATEGIES[resolved.ref.kind];
  if (!strategy) return false;
  return strategy(resolved, bridge);
}

type OpenTargetStrategy = (
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
) => boolean;

const OPEN_TARGET_STRATEGIES: Record<EditorTargetRef["kind"], OpenTargetStrategy> = {
  mod: (_resolved, bridge) => openComponent(bridge, ProjectOverviewComponent),
  projectNode: (resolved, bridge) => openProjectNodeTarget(resolved, bridge),
  projectFile: (resolved, bridge) => openProjectFileTarget(resolved, bridge),
  script: (resolved, bridge) => openScriptTarget(resolved, bridge),
  asset: (resolved, bridge) => openAssetTarget(resolved, bridge),
  scene: (resolved, bridge) => openSceneTarget(resolved, bridge),
  sceneEntity: (resolved, bridge) => openSceneEntityTarget(resolved, bridge),
  component: (resolved, bridge) => openComponentTarget(resolved, bridge),
  uiDocument: (resolved, bridge) => openUiDocumentTarget(resolved, bridge),
  uiNode: (resolved, bridge) => openUiNodeTarget(resolved, bridge),
  diagnostic: (_resolved, bridge) => openComponent(bridge, DiagnosticsProblemsComponent),
  capability: (_resolved, bridge) => openComponent(bridge, ProjectCapabilitiesComponent),
  dependency: (_resolved, bridge) => openComponent(bridge, ProjectDependenciesComponent),
};

export function revealResolvedEditorTarget(
  resolved: ResolvedEditorTarget,
  services: WorkspaceRuntimeServices,
): boolean {
  const { ref, selection } = resolved;
  const bridge = services.targetBridge;

  if ((ref.kind === "projectFile" || ref.kind === "script") && selection.kind === "projectFile") {
    bridge?.handleSelectProjectFile?.(selection.file);
    return true;
  }

  if (ref.kind === "asset" && selection.kind === "asset" && selection.file) {
    bridge?.handleSelectProjectFile?.(selection.file);
    return true;
  }

  if (ref.kind === "scene" && selection.kind === "scene") {
    void bridge?.openSceneEditor?.(selection.scene);
    return true;
  }

  if (ref.kind === "diagnostic" && ref.path) {
    const file = findProjectFileByPath(services, ref.path);
    if (file) {
      bridge?.handleSelectProjectFile?.(file);
      return true;
    }
  }

  return false;
}

function openComponent(bridge: EditorTargetRuntimeBridge | undefined, component: EditorComponentDefinition<any>) {
  if (!component) return false;
  if (bridge?.openComponent) {
    bridge.openComponent({ component });
    return true;
  }

  openWorkspaceEditor(bridge, {
    kind: "component",
    component,
  });
  return true;
}

function openUiDocument(
  bridge: EditorTargetRuntimeBridge | undefined,
  request: {
    sceneId: string;
    entityId: string;
    componentIndex: number;
    focusPath?: string;
    titleOverride?: string;
  },
) {
  if (bridge?.openUiDocumentEditor) {
    bridge.openUiDocumentEditor(request);
    return true;
  }

  openWorkspaceEditor(bridge, {
    kind: "ui-document",
    ...request,
  });
  return true;
}

function openWorkspaceEditor(
  bridge: EditorTargetRuntimeBridge | undefined,
  request: OpenWorkspaceEditorRequest,
) {
  bridge?.openWorkspaceEditor?.(request);
  return true;
}

function openProjectNodeTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.ref.kind !== "projectNode") return false;

  const component = projectNodeComponentForKind(resolved.ref.nodeKind);
  if (!component) return false;
  if (component === DiagnosticsProblemsComponent) {
    bridge?.showBottomComponent?.(DiagnosticsProblemsComponent);
  }
  return openComponent(bridge, component);
}

function projectNodeComponentForKind(nodeKind: string): EditorComponentDefinition<any> | null {
  const componentId = {
    modRoot: ProjectOverviewComponent.id,
    overview: ProjectOverviewComponent.id,
    capabilities: ProjectCapabilitiesComponent.id,
    dependencies: ProjectDependenciesComponent.id,
    diagnostics: DiagnosticsProblemsComponent.id,
  }[nodeKind];
  return componentId ? (editorComponentById(componentId) ?? null) : null;
}

function openProjectFileTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.selection.kind !== "projectFile") return false;
  return openWorkspaceEditor(bridge, { kind: "project-file", file: resolved.selection.file });
}

function openScriptTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.selection.kind !== "projectFile") return false;
  bridge?.openProjectFileEditor?.(resolved.selection.file);
  return true;
}

function openAssetTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.selection.kind !== "asset") return false;
  return openWorkspaceEditor(bridge, { kind: "asset", asset: resolved.selection.asset });
}

function openSceneTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.selection.kind !== "scene") return false;
  if (bridge?.openSceneEditor) {
    void bridge.openSceneEditor(resolved.selection.scene);
    return true;
  }
  return openWorkspaceEditor(bridge, { kind: "scene", scene: resolved.selection.scene });
}

function openSceneEntityTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.ref.kind !== "sceneEntity") return false;
  bridge?.selectSceneEntity?.(resolved.ref.entityId);
  return true;
}

function openComponentTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.ref.kind !== "component") return false;
  bridge?.selectSceneEntity?.(resolved.ref.entityId);
  return true;
}

function openUiDocumentTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.ref.kind !== "uiDocument") return false;
  const component = editorComponentById(UiDocumentEditorComponent.id) ?? UiDocumentEditorComponent;
  openComponent(bridge, component);
  return openUiDocument(bridge, {
    sceneId: resolved.ref.sceneId,
    entityId: resolved.ref.entityId,
    componentIndex: resolved.ref.componentIndex,
  });
}

function openUiNodeTarget(
  resolved: ResolvedEditorTarget,
  bridge: EditorTargetRuntimeBridge | undefined,
): boolean {
  if (resolved.ref.kind !== "uiNode") return false;
  bridge?.selectUiNode?.({
    entityId: resolved.ref.entityId,
    componentIndex: resolved.ref.componentIndex,
    nodePath: resolved.ref.nodePath,
  });
  return openUiDocument(bridge, {
    sceneId: resolved.ref.sceneId,
    entityId: resolved.ref.entityId,
    componentIndex: resolved.ref.componentIndex,
    focusPath: resolved.ref.nodePath,
    titleOverride: resolved.descriptor.label,
  });
}

function findProjectFileByPath(services: WorkspaceRuntimeServices, path: string) {
  const root = services.projectTree?.root;
  if (!root) return null;
  const normalized = normalizePath(path);

  function visit(file: NonNullable<typeof root>): NonNullable<typeof root> | null {
    if (
      normalizePath(file.relativePath) === normalized ||
      normalizePath(file.path) === normalized
    ) {
      return file;
    }

    for (const child of file.children ?? []) {
      const found = visit(child);
      if (found) return found;
    }

    return null;
  }

  return visit(root);
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\/+/, "").toLowerCase();
}
