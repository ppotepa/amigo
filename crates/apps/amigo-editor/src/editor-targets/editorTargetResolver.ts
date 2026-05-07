import type {
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
  ManagedAssetDto,
} from "../api/dto";
import type { WorkspaceRuntimeServices } from "../main-window/workspaceRuntimeServices";
import type { EditorSelection } from "../properties/propertiesTypes";
import {
  editorTargetKey,
  emptyEditorTargetSelection,
  type EditorTargetAction,
  type EditorTargetDescriptor,
  type EditorTargetRef,
  type ResolvedEditorTarget,
} from "./editorTargetTypes";
import { editorTargetContextProfileFor } from "./editorTargetContextProfiles";

// @codemap anchor:editor-target-resolver domain:workspace role:dispatcher priority:P1 layer:app tags:editor-target,selection,right-dock
export function resolveEditorTarget(
  target: EditorTargetRef,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  switch (target.kind) {
    case "mod":
      return resolveModTarget(target, services);
    case "projectNode":
      return resolveProjectNodeTarget(target);
    case "projectFile":
      return resolveProjectFileTarget(target.path, services, "projectFile");
    case "script":
      return resolveProjectFileTarget(target.path, services, "script");
    case "asset":
      return resolveAssetTarget(target.assetKey, services);
    case "scene":
      return resolveSceneTarget(target.sceneId, services);
    case "sceneEntity":
      return resolveSceneEntityTarget(target.sceneId, target.entityId, services);
    case "uiDocument":
      return resolveUiDocumentTarget(target, services);
    case "uiNode":
      return resolveUiNodeTarget(target, services);
    case "diagnostic":
      return resolveDiagnosticTarget(target, services);
    case "capability":
      return resolveCapabilityTarget(target.capabilityId, services);
    case "dependency":
      return resolveDependencyTarget(target.dependencyId, services);
  }
}

function resolveModTarget(
  target: Extract<EditorTargetRef, { kind: "mod" }>,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const details = services.details ?? null;
  const label = details?.name || target.modId;

  return resolved({
    ref: target,
    label,
    subtitle: details?.id ?? target.modId,
    icon: "Boxes",
    breadcrumbs: [label],
    selection: { kind: "mod", details },
    canOpen: true,
    actions: [
      action("open", "Open overview", "primary"),
      action("validate", "Validate", "default"),
    ],
  });
}

function resolveProjectNodeTarget(
  target: Extract<EditorTargetRef, { kind: "projectNode" }>,
): ResolvedEditorTarget {
  const label = target.label || humanize(target.nodeKind || target.nodeId);
  const canOpen =
    target.nodeKind === "modRoot" ||
    target.nodeKind === "overview" ||
    target.nodeKind === "capabilities" ||
    target.nodeKind === "dependencies" ||
    target.nodeKind === "diagnostics" ||
    target.nodeKind === "manifest";

  return resolved({
    ref: target,
    label,
    subtitle: target.path ?? target.expectedPath ?? target.nodeKind,
    icon: "FolderTree",
    breadcrumbs: ["Project", label],
    selection: emptyEditorTargetSelection(),
    canOpen,
    actions: canOpen ? [action("open", "Open", "primary")] : [],
  });
}

function resolveProjectFileTarget(
  path: string,
  services: WorkspaceRuntimeServices,
  sourceKind: "projectFile" | "script",
): ResolvedEditorTarget {
  const file = findProjectFileByPath(services.projectTree?.root, path);
  const label = file?.name ?? basename(path);
  const target: EditorTargetRef = sourceKind === "script"
    ? { kind: "script", path }
    : { kind: "projectFile", path };

  if (!file) {
    return missing({
      ref: target,
      label,
      subtitle: path,
      icon: sourceKind === "script" ? "Code2" : "FileText",
      breadcrumbs: ["Files", label],
      reason: `Project file not found: ${path}`,
    });
  }

  return resolved({
    ref: target,
    label,
    subtitle: file.relativePath,
    icon: sourceKind === "script" ? "Code2" : "FileText",
    breadcrumbs: ["Files", file.relativePath],
    selection: { kind: "projectFile", file },
    canOpen: !file.isDir,
    actions: [
      action("open", "Open", "primary"),
      action("reveal", "Reveal"),
      action("showYaml", "Show source"),
    ],
  });
}

function resolveAssetTarget(
  assetKey: string,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const asset = findManagedAsset(services, assetKey);
  const target: EditorTargetRef = { kind: "asset", assetKey };

  if (!asset) {
    return missing({
      ref: target,
      label: assetKey,
      subtitle: "Missing asset",
      icon: "PackageX",
      breadcrumbs: ["Assets", assetKey],
      reason: `Asset not found: ${assetKey}`,
    });
  }

  const file = findProjectFileByPath(services.projectTree?.root, asset.descriptorRelativePath);

  return resolved({
    ref: target,
    label: asset.label || asset.assetKey,
    subtitle: `${asset.kind} · ${asset.descriptorRelativePath}`,
    icon: "Package",
    breadcrumbs: ["Assets", asset.domain, asset.label || asset.assetKey],
    selection: { kind: "asset", asset, file },
    canOpen: true,
    actions: [
      action("open", "Open", "primary"),
      action("reveal", "Reveal"),
      action("showYaml", "Show YAML"),
    ],
  });
}

function resolveSceneTarget(
  sceneId: string,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const scene = findScene(services, sceneId);
  const target: EditorTargetRef = { kind: "scene", sceneId };

  if (!scene) {
    return missing({
      ref: target,
      label: sceneId,
      subtitle: "Missing scene",
      icon: "MonitorX",
      breadcrumbs: ["Scenes", sceneId],
      reason: `Scene not found: ${sceneId}`,
    });
  }

  return resolved({
    ref: target,
    label: scene.label || scene.id,
    subtitle: scene.documentPath,
    icon: "Monitor",
    breadcrumbs: ["Scenes", scene.label || scene.id],
    selection: { kind: "scene", scene },
    canOpen: true,
    actions: [
      action("open", "Open scene", "primary"),
      action("reveal", "Reveal"),
    ],
  });
}

function resolveSceneEntityTarget(
  sceneId: string,
  entityId: string,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const scene = findScene(services, sceneId) ?? services.selectedScene ?? null;
  const entity = findEntity(services, entityId);
  const target: EditorTargetRef = { kind: "sceneEntity", sceneId, entityId };

  if (!entity) {
    return missing({
      ref: target,
      label: entityId,
      subtitle: "Missing scene entity",
      icon: "Box",
      breadcrumbs: ["Scenes", sceneId, entityId],
      reason: `Scene entity not found: ${entityId}`,
    });
  }

  return resolved({
    ref: target,
    label: entity.name || entity.id,
    subtitle: entity.componentTypes.join(", ") || "Entity",
    icon: "Box",
    breadcrumbs: ["Scenes", scene?.label ?? sceneId, entity.name || entity.id],
          selection: {
            kind: "entity",
            scene,
            entity,
          },
    canOpen: true,
    actions: [
      action("focusViewport", "Focus in viewport", "primary"),
      action("reveal", "Reveal"),
    ],
  });
}

function resolveUiDocumentTarget(
  target: Extract<EditorTargetRef, { kind: "uiDocument" }>,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const document = findUiDocument(services, target.entityId, target.componentIndex);
  const scene = findScene(services, target.sceneId) ?? services.selectedScene ?? null;

  if (!document) {
    return missing({
      ref: target,
      label: "UI Document",
      subtitle: `${target.entityId} #${target.componentIndex}`,
      icon: "LayoutPanelTop",
      breadcrumbs: ["UI", target.entityId],
      reason: `UI document not found: ${target.entityId} #${target.componentIndex}`,
    });
  }

  return resolved({
    ref: target,
    label: `${document.entityName} UI`,
    subtitle: `Component #${document.componentIndex}`,
    icon: "LayoutPanelTop",
    breadcrumbs: ["Scenes", scene?.label ?? target.sceneId, document.entityName, "UI Document"],
    selection: emptyEditorTargetSelection(),
    canOpen: true,
    actions: [
      action("open", "Open UI document", "primary"),
      action("reveal", "Reveal"),
    ],
  });
}

function resolveUiNodeTarget(
  target: Extract<EditorTargetRef, { kind: "uiNode" }>,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const scene = findScene(services, target.sceneId) ?? services.selectedScene ?? null;
  const entity = findEntity(services, target.entityId);
  const document = findUiDocument(services, target.entityId, target.componentIndex);
  const node = document ? findUiNode(document.root, target.nodePath) : null;

  if (!document || !entity || !node) {
    return missing({
      ref: target,
      label: target.nodePath,
      subtitle: "Missing UI node",
      icon: "MousePointerClick",
      breadcrumbs: ["UI", target.nodePath],
      reason: `UI node not found: ${target.nodePath}`,
    });
  }

  return resolved({
    ref: target,
    label: node.label || node.id,
    subtitle: `${node.kind} · ${node.path}`,
    icon: "MousePointerClick",
    breadcrumbs: ["Scenes", scene?.label ?? target.sceneId, document.entityName, node.path],
    selection: {
      kind: "uiNode",
      scene,
      entity,
      node,
      nodeRef: {
        entityId: target.entityId,
        componentIndex: target.componentIndex,
        nodePath: target.nodePath,
      },
    },
    canOpen: true,
    actions: [
      action("open", "Open focused UI view", "primary"),
      action("reveal", "Reveal"),
    ],
  });
}

function resolveDiagnosticTarget(
  target: Extract<EditorTargetRef, { kind: "diagnostic" }>,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const diagnostic = findDiagnostic(services, target);
  const label = diagnostic?.code ?? target.code ?? "Diagnostic";
  const subtitle = diagnostic?.message ?? target.path ?? target.diagnosticId;

  return {
    ref: target,
    status: diagnostic ? "resolved" : "missing",
    reason: diagnostic ? undefined : `Diagnostic not found: ${target.diagnosticId}`,
    selection: emptyEditorTargetSelection(),
    contextProfile: editorTargetContextProfileFor(target.kind),
    descriptor: {
      kind: "diagnostic",
      label,
      subtitle,
      icon: diagnostic?.level === "error" ? "CircleX" : "TriangleAlert",
      breadcrumbs: ["Diagnostics", label],
      canOpen: Boolean(diagnostic?.path),
      canReveal: Boolean(diagnostic?.path),
      canInspect: true,
      selectionKind: "empty",
      actions: diagnostic?.path ? [action("reveal", "Reveal source", "primary")] : [],
    },
  };
}

function resolveCapabilityTarget(
  capabilityId: string,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  return resolved({
    ref: { kind: "capability", capabilityId },
    label: capabilityId,
    subtitle: "Capability",
    icon: "Plug",
    breadcrumbs: ["Project", "Capabilities", capabilityId],
    selection: { kind: "mod", details: services.details ?? null },
    canOpen: true,
    actions: [
      action("open", "Open capabilities", "primary"),
      action("inspect", "Inspect"),
    ],
  });
}

function resolveDependencyTarget(
  dependencyId: string,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  return resolved({
    ref: { kind: "dependency", dependencyId },
    label: dependencyId,
    subtitle: "Dependency",
    icon: "Network",
    breadcrumbs: ["Project", "Dependencies", dependencyId],
    selection: { kind: "mod", details: services.details ?? null },
    canOpen: true,
    actions: [
      action("open", "Open dependencies", "primary"),
      action("inspect", "Inspect"),
    ],
  });
}

function resolved({
  ref,
  label,
  subtitle,
  icon,
  breadcrumbs,
  selection,
  canOpen,
  actions,
}: {
  ref: EditorTargetRef;
  label: string;
  subtitle?: string;
  icon: string;
  breadcrumbs: string[];
  selection: EditorSelection;
  canOpen: boolean;
  actions: EditorTargetAction[];
}): ResolvedEditorTarget {
  return {
    ref,
    status: "resolved",
    selection,
    contextProfile: editorTargetContextProfileFor(ref.kind),
    descriptor: {
      kind: ref.kind,
      label,
      subtitle,
      icon,
      breadcrumbs,
      canOpen,
      canReveal: true,
      canInspect: true,
      selectionKind: selection.kind,
      actions,
    },
  };
}

function missing({
  ref,
  label,
  subtitle,
  icon,
  breadcrumbs,
  reason,
}: {
  ref: EditorTargetRef;
  label: string;
  subtitle?: string;
  icon: string;
  breadcrumbs: string[];
  reason: string;
}): ResolvedEditorTarget {
  return {
    ref,
    status: "missing",
    reason,
    selection: emptyEditorTargetSelection(),
    contextProfile: editorTargetContextProfileFor(ref.kind),
    descriptor: {
      kind: ref.kind,
      label,
      subtitle,
      icon,
      breadcrumbs,
      canOpen: false,
      canReveal: false,
      canInspect: true,
      selectionKind: "empty",
      actions: [],
    },
  };
}

function action(
  id: string,
  label: string,
  tone: EditorTargetAction["tone"] = "default",
): EditorTargetAction {
  return {
    id,
    label,
    tone,
    enabled: true,
    visible: true,
  };
}

function findManagedAsset(
  services: WorkspaceRuntimeServices,
  assetKey: string,
): ManagedAssetDto | null {
  return services.assetRegistry?.managedAssets.find((asset) => asset.assetKey === assetKey) ?? null;
}

function findScene(
  services: WorkspaceRuntimeServices,
  sceneId: string,
): EditorSceneSummaryDto | null {
  if (services.selectedScene?.id === sceneId) return services.selectedScene;
  return services.details?.scenes.find((scene) => scene.id === sceneId) ?? null;
}

function findEntity(
  services: WorkspaceRuntimeServices,
  entityId: string,
): EditorSceneEntityDto | null {
  if (services.selectedEntity?.id === entityId) return services.selectedEntity;
  return services.hierarchy?.entities.find((entity) => entity.id === entityId) ?? null;
}

function findUiDocument(
  services: WorkspaceRuntimeServices,
  entityId: string,
  componentIndex: number,
): EditorUiDocumentDto | null {
  return services.hierarchy?.uiDocuments.find(
    (document) => document.entityId === entityId && document.componentIndex === componentIndex,
  ) ?? null;
}

function findUiNode(root: EditorUiNodeDto, path: string): EditorUiNodeDto | null {
  if (root.path === path) return root;
  for (const child of root.children) {
    const found = findUiNode(child, path);
    if (found) return found;
  }
  return null;
}

function findProjectFileByPath(
  root: EditorProjectFileDto | null | undefined,
  path: string | null | undefined,
): EditorProjectFileDto | null {
  if (!root || !path) return null;
  const normalizedPath = normalizePath(path);

  function visit(file: EditorProjectFileDto): EditorProjectFileDto | null {
    if (
      normalizePath(file.relativePath) === normalizedPath ||
      normalizePath(file.path) === normalizedPath ||
      normalizePath(file.name) === normalizedPath
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

function findDiagnostic(
  services: WorkspaceRuntimeServices,
  target: Extract<EditorTargetRef, { kind: "diagnostic" }>,
): EditorDiagnosticDto | null {
  const diagnostics = services.allProblems ?? [];
  return diagnostics.find((diagnostic, index) => {
    const candidate = [
      diagnostic.level,
      diagnostic.code,
      diagnostic.path ?? "",
      diagnostic.message,
      String(index),
    ].join(":");

    return (
      candidate === target.diagnosticId ||
      diagnostic.code === target.code ||
      (Boolean(target.path) && diagnostic.path === target.path)
    );
  }) ?? null;
}

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\/+/, "").toLowerCase();
}

function basename(path: string): string {
  return path.replace(/\\/g, "/").split("/").filter(Boolean).pop() ?? path;
}

function humanize(value: string): string {
  return value
    .replace(/[-_]+/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
    .trim();
}
