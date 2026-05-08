import {
  DiagnosticsProblemsComponent,
  ProjectCapabilitiesComponent,
  ProjectDependenciesComponent,
  ProjectOverviewComponent,
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

  const { ref, selection } = resolved;

  if (ref.kind === "mod") {
    openComponent(bridge, ProjectOverviewComponent);
    return true;
  }

  if (ref.kind === "projectNode") {
    if (ref.nodeKind === "modRoot" || ref.nodeKind === "overview") {
      openComponent(bridge, ProjectOverviewComponent);
      return true;
    }
    if (ref.nodeKind === "capabilities") {
      openComponent(bridge, ProjectCapabilitiesComponent);
      return true;
    }
    if (ref.nodeKind === "dependencies") {
      openComponent(bridge, ProjectDependenciesComponent);
      return true;
    }
    if (ref.nodeKind === "diagnostics") {
      openComponent(bridge, DiagnosticsProblemsComponent);
      bridge?.showBottomComponent?.(DiagnosticsProblemsComponent);
      return true;
    }
  }

  if ((ref.kind === "projectFile" || ref.kind === "script") && selection.kind === "projectFile") {
    if (ref.kind === "script") {
      bridge?.openProjectFileEditor?.(selection.file);
      return true;
    }
    openWorkspaceEditor(bridge, { kind: "project-file", file: selection.file });
    return true;
  }

  if (ref.kind === "asset" && selection.kind === "asset") {
    openWorkspaceEditor(bridge, { kind: "asset", asset: selection.asset });
    return true;
  }

  if (ref.kind === "scene" && selection.kind === "scene") {
    if (bridge?.openSceneEditor) {
      void bridge.openSceneEditor(selection.scene);
    } else {
      openWorkspaceEditor(bridge, { kind: "scene", scene: selection.scene });
    }
    return true;
  }

  if (ref.kind === "sceneEntity") {
    bridge?.selectSceneEntity?.(ref.entityId);
    return true;
  }

  if (ref.kind === "uiDocument") {
    openUiDocument(bridge, {
      sceneId: ref.sceneId,
      entityId: ref.entityId,
      componentIndex: ref.componentIndex,
    });
    return true;
  }

  if (ref.kind === "uiNode") {
    bridge?.selectUiNode?.({
      entityId: ref.entityId,
      componentIndex: ref.componentIndex,
      nodePath: ref.nodePath,
    });
    openUiDocument(bridge, {
      sceneId: ref.sceneId,
      entityId: ref.entityId,
      componentIndex: ref.componentIndex,
      focusPath: ref.nodePath,
      titleOverride: resolved.descriptor.label,
    });
    return true;
  }

  if (ref.kind === "capability") {
    openComponent(bridge, ProjectCapabilitiesComponent);
    return true;
  }

  if (ref.kind === "dependency") {
    openComponent(bridge, ProjectDependenciesComponent);
    return true;
  }

  return false;
}

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
  if (bridge?.openComponent) {
    bridge.openComponent({ component });
    return;
  }

  openWorkspaceEditor(bridge, {
    kind: "component",
    component,
  });
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
    return;
  }

  openWorkspaceEditor(bridge, {
    kind: "ui-document",
    ...request,
  });
}

function openWorkspaceEditor(
  bridge: EditorTargetRuntimeBridge | undefined,
  request: OpenWorkspaceEditorRequest,
) {
  bridge?.openWorkspaceEditor?.(request);
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
