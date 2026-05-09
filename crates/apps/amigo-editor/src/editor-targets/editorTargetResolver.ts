import type {
  EditorDiagnosticDto,
  EditorProjectFileDto,
  EditorSceneComponentInstanceDto,
  EditorSceneEntityDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
  ManagedAssetDto,
} from "../api/dto";
import type { WorkspaceRuntimeServices } from "../main-window/workspaceRuntimeServices";
import type { EditorSelection } from "../properties/propertiesTypes";
import {
  emptyEditorTargetSelection,
  type EditorTargetAction,
  type EditorTargetDescriptor,
  type EditorTargetDiagnosticRef,
  type EditorTargetDocumentRef,
  type EditorTargetIntent,
  type EditorTargetMetadataRef,
  type EditorTargetRef,
  type EditorTargetRelatedRef,
  type ResolvedEditorTarget,
} from "./editorTargetTypes";

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
    case "component":
      return resolveComponentTarget(target, services);
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
  const capabilities = details?.capabilities ?? [];

  return resolved({
    ref: target,
    label,
    subtitle: details?.id ?? target.modId,
    icon: "Boxes",
    breadcrumbs: ["Project", label],
    selection: { kind: "mod", details },
    canOpen: true,
    capabilities: ["project", "mod", "inspectable", ...capabilities],
    metadataRefs: [
      metadataRef("targetKind", "mod", "Mod target"),
      ...capabilities.map((capability) => metadataRef("capability", capability, capability, "declared")),
    ],
    documentRefs: details?.rootPath
      ? [documentRef("directory", "Project root", details.rootPath, "root", undefined, true)]
      : [],
    relatedTargets: [
      relatedRef("capabilities", "Capabilities", { kind: "projectNode", nodeId: "capabilities", nodeKind: "capabilities", label: "Capabilities" }),
      relatedRef("dependencies", "Dependencies", { kind: "projectNode", nodeId: "dependencies", nodeKind: "dependencies", label: "Dependencies" }),
      relatedRef("diagnostics", "Diagnostics", { kind: "projectNode", nodeId: "diagnostics", nodeKind: "diagnostics", label: "Diagnostics" }),
    ],
    diagnostics: diagnosticRefs(details?.diagnostics ?? []),
    actions: [
      action("open", "Open overview", "primary", "open"),
      action("validate", "Validate", "default", "inspect"),
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

  const path = target.path ?? target.expectedPath ?? null;

  return resolved({
    ref: target,
    label,
    subtitle: path ?? target.nodeKind,
    icon: "FolderTree",
    breadcrumbs: ["Project", label],
    selection: emptyEditorTargetSelection(),
    canOpen,
    capabilities: ["project-node", "navigable", canOpen ? "openable" : "inspectable"],
    metadataRefs: [
      metadataRef("targetKind", "projectNode", "Project node"),
      metadataRef("custom", target.nodeKind || target.nodeId, label, "nodeKind"),
    ],
    documentRefs: path
      ? [documentRef("projectFile", label, path, "project-node-source", { kind: "projectFile", path }, true)]
      : [],
    relatedTargets: [],
    diagnostics: [],
    actions: canOpen ? [action("open", "Open", "primary", "open")] : [],
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
      documentRefs: [documentRef("projectFile", label, path, "missing-project-file", target, true)],
      diagnostics: diagnosticsForTarget(services, [path], [label]),
    });
  }

  const documentKind = file.isDir ? "directory" : "projectFile";
  const capabilities = [
    file.isDir ? "directory" : "file",
    file.isDir ? "navigable" : "openable",
    "revealable",
    "diagnostic-host",
    sourceKind === "script" ? "script" : "project-file",
  ];

  return resolved({
    ref: target,
    label,
    subtitle: file.relativePath,
    icon: sourceKind === "script" ? "Code2" : "FileText",
    breadcrumbs: ["Files", file.relativePath],
    selection: { kind: "projectFile", file },
    canOpen: !file.isDir,
    capabilities,
    metadataRefs: [
      metadataRef("targetKind", sourceKind, sourceKind === "script" ? "Script" : "Project file"),
      metadataRef("documentKind", file.kind, humanize(file.kind), "project-file-kind"),
    ],
    documentRefs: [
      documentRef(documentKind, file.name, file.relativePath, sourceKind, target, file.isDir),
    ],
    relatedTargets: [],
    diagnostics: diagnosticsForTarget(services, [file.relativePath, file.path], [label]),
    actions: [
      action("open", "Open", "primary", "open"),
      action("reveal", "Reveal", "default", "reveal"),
      action("showYaml", "Show source", "default", "open"),
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
      metadataRefs: [metadataRef("targetKind", "asset", "Asset target")],
    });
  }

  const file = findProjectFileByPath(services.projectTree?.root, asset.descriptorRelativePath);
  const sourceFileRefs = asset.sourceFiles.map((source) =>
    documentRef(
      "assetSource",
      basename(source.relativePath),
      source.relativePath,
      source.role || "asset-source",
      { kind: "projectFile", path: source.relativePath },
      true,
    ),
  );

  return resolved({
    ref: target,
    label: asset.label || asset.assetKey,
    subtitle: `${asset.kind} - ${asset.descriptorRelativePath}`,
    icon: "Package",
    breadcrumbs: ["Assets", asset.domain, asset.label || asset.assetKey],
    selection: { kind: "asset", asset, file },
    canOpen: true,
    capabilities: [
      "asset",
      "previewable",
      "openable",
      "revealable",
      asset.usedBy.length ? "has-usages" : "unused",
      asset.sourceFiles.length ? "has-source-files" : "descriptor-only",
    ],
    metadataRefs: [
      metadataRef("targetKind", "asset", "Asset target"),
      metadataRef("assetKind", asset.kind, humanize(asset.kind), "kind"),
      metadataRef("custom", asset.domain, asset.domain, "domain"),
      metadataRef("custom", asset.role, asset.role, "role"),
    ],
    documentRefs: [
      documentRef("assetDescriptor", "Asset descriptor", asset.descriptorRelativePath, "descriptor", { kind: "projectFile", path: asset.descriptorRelativePath }),
      ...sourceFileRefs,
    ],
    relatedTargets: [
      ...(file ? [relatedRef("descriptor-file", file.relativePath, { kind: "projectFile", path: file.relativePath })] : []),
      ...asset.usedBy.slice(0, 8).map((usage) => relatedRef("used-by", usage, undefined, "asset usage")),
      ...asset.references.slice(0, 8).map((reference) => relatedRef("references", reference, { kind: "asset", assetKey: reference })),
    ],
    diagnostics: [
      ...diagnosticRefs(asset.diagnostics),
      ...diagnosticsForTarget(services, [asset.descriptorRelativePath, ...asset.sourceFiles.map((source) => source.relativePath)], [asset.assetKey, asset.label]),
    ],
    actions: [
      action("open", "Open", "primary", "open"),
      action("reveal", "Reveal", "default", "reveal"),
      action("showYaml", "Show YAML", "default", "open"),
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
      metadataRefs: [metadataRef("targetKind", "scene", "Scene target")],
    });
  }

  const entityTargets =
    services.hierarchy?.sceneId === scene.id
      ? services.hierarchy.entities.slice(0, 12).map((entity) =>
          relatedRef("entity", entity.name || entity.id, { kind: "sceneEntity", sceneId: scene.id, entityId: entity.id }),
        )
      : [];

  return resolved({
    ref: target,
    label: scene.label || scene.id,
    subtitle: scene.documentPath,
    icon: "Monitor",
    breadcrumbs: ["Scenes", scene.label || scene.id],
    selection: { kind: "scene", scene },
    canOpen: true,
    capabilities: [
      "scene",
      "openable",
      "previewable",
      "yaml-document",
      scene.launcherVisible ? "launcher-visible" : "launcher-hidden",
    ],
    metadataTraits: [
      "SceneDocument",
      "HasEntities",
      "HasScripts",
      "HasDiagnostics",
      "HasAssetUsages",
      "HasUiDocuments",
    ],
    metadataRefs: [
      metadataRef("targetKind", "scene", "Scene target"),
      metadataRef("documentKind", "sceneYaml", "Scene YAML"),
    ],
    documentRefs: [
      documentRef("sceneYaml", "Scene YAML", scene.documentPath, "scene-document", { kind: "projectFile", path: scene.documentPath }),
      ...(scene.scriptPath ? [documentRef("sceneScript", "Scene script", scene.scriptPath, "scene-script", { kind: "script", path: scene.scriptPath })] : []),
    ],
    relatedTargets: entityTargets,
    diagnostics: [
      ...diagnosticRefs(scene.diagnostics),
      ...diagnosticsForTarget(services, [scene.documentPath, scene.scriptPath], [scene.id, scene.label]),
    ],
    actions: [
      action("open", "Open scene", "primary", "open"),
      action("reveal", "Reveal", "default", "reveal"),
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
      documentRefs: scene?.documentPath
        ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId })]
        : [],
      metadataRefs: [metadataRef("targetKind", "sceneEntity", "Scene entity target")],
    });
  }

  const componentTypes = entity.components?.length
    ? entity.components.map((component) => component.typeName)
    : entity.componentTypes;

  const componentTargets = (entity.components ?? []).map((component) =>
    relatedRef(
      "component",
      component.label || component.typeName,
      {
        kind: "component",
        sceneId,
        entityId: entity.id,
        componentIndex: component.componentIndex,
        componentType: component.typeName,
      },
      `#${component.componentIndex}`,
    ),
  );

  const capabilities = [
    "scene-entity",
    "selectable",
    "focusable",
    entity.visible ? "visible" : "hidden",
    entity.simulationEnabled ? "simulation-enabled" : "simulation-disabled",
    entity.collisionEnabled ? "collision-enabled" : "collision-disabled",
    entity.hasTransform2 ? "transform-2d" : "",
    entity.hasTransform3 ? "transform-3d" : "",
  ].filter(Boolean);
  const metadataTraits = entity.metadataTraits ?? [
    ...(entity.ownTraits ?? []),
    ...(entity.derivedTraits ?? []),
  ];

  return resolved({
    ref: target,
    label: entity.name || entity.id,
    subtitle: componentTypes.join(", ") || "Entity",
    icon: "Box",
    breadcrumbs: ["Scenes", scene?.label ?? sceneId, entity.name || entity.id],
    selection: {
      kind: "entity",
      scene,
      entity,
    },
    canOpen: true,
    capabilities,
    metadataTraits,
    metadataRefs: [
      metadataRef("targetKind", "sceneEntity", "Scene entity target"),
      ...metadataTraits.map((traitKind) =>
        metadataRef("metadataTrait", traitKind, traitKind, "entity-trait"),
      ),
      ...componentTypes.map((componentType) => metadataRef("component", componentType, componentType, "attached")),
    ],
    documentRefs: scene?.documentPath
      ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId })]
      : [],
    relatedTargets: [
      ...(scene ? [relatedRef("scene", scene.label || scene.id, { kind: "scene", sceneId: scene.id }, "owner scene")] : []),
      ...componentTargets,
    ],
    diagnostics: diagnosticsForTarget(services, [scene?.documentPath], [entity.id, entity.name]),
    actions: [
      action("focusViewport", "Focus in viewport", "primary", "inspect"),
      action("reveal", "Reveal", "default", "reveal"),
    ],
  });
}

// @codemap anchor:component-target-resolver domain:workspace role:dispatcher priority:P1 layer:app tags:editor-target,component,metadata
function resolveComponentTarget(
  target: Extract<EditorTargetRef, { kind: "component" }>,
  services: WorkspaceRuntimeServices,
): ResolvedEditorTarget {
  const scene = findScene(services, target.sceneId) ?? services.selectedScene ?? null;
  if (target.ownerKind === "scene" || !target.entityId) {
    const component = findSceneOwnedComponent(services, target);
    const componentLabel = component?.label || target.componentType;
    return resolved({
      ref: target,
      label: componentLabel,
      subtitle: component?.yamlPath ?? `Scene component #${target.componentIndex}`,
      icon: "Puzzle",
      breadcrumbs: ["Scenes", scene?.label ?? target.sceneId, componentLabel],
      selection: scene ? { kind: "scene", scene } : emptyEditorTargetSelection(),
      canOpen: false,
      capabilities: ["component", "scene-owned", "inspectable", "metadata-backed"],
      metadataRefs: [
        metadataRef("targetKind", "component", "Component target"),
        metadataRef("component", target.componentType, target.componentType, "scene-owned"),
      ],
      documentRefs: scene?.documentPath
        ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
        : [],
      relatedTargets: scene
        ? [relatedRef("scene", scene.label || scene.id, { kind: "scene", sceneId: scene.id }, "owner scene")]
        : [],
      diagnostics: diagnosticsForTarget(services, [scene?.documentPath], [target.componentType]),
      actions: [
        action("inspect", "Inspect component", "primary", "inspect"),
        action("reveal", "Reveal", "default", "reveal"),
      ],
    });
  }

  const entity = findEntity(services, target.entityId);
  const component = entity ? findComponent(entity, target.componentIndex, target.componentType) : null;

  if (!entity || !component) {
    return missing({
      ref: target,
      label: target.componentType,
      subtitle: "Missing component",
      icon: "Puzzle",
      breadcrumbs: ["Scenes", target.sceneId, target.entityId ?? "scene", target.componentType],
      reason: `Component not found: ${target.entityId ?? "scene"} #${target.componentIndex} ${target.componentType}`,
      documentRefs: scene?.documentPath
        ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
        : [],
      metadataRefs: [
        metadataRef("targetKind", "component", "Component target"),
        metadataRef("component", target.componentType, target.componentType, "missing"),
      ],
    });
  }

  const componentLabel = component.label || component.typeName;
  const assetTargets = component.assetRefs
    .filter((ref) => Boolean(ref.value))
    .map((ref) =>
      relatedRef(
        "asset-ref",
        ref.value ?? ref.fieldPath,
        ref.value ? { kind: "asset", assetKey: ref.value } : undefined,
        `${ref.fieldPath} (${ref.domain})`,
      ),
    );
  const descriptorCapabilities = component.descriptorKind ? [`descriptor:${component.descriptorKind}`] : [];
  const metadataTraits = component.metadataTraits ?? [];

  return resolved({
    ref: target,
    label: componentLabel,
    subtitle: `${entity.name || entity.id} / #${component.componentIndex} / ${component.yamlPath}`,
    icon: "Puzzle",
    breadcrumbs: ["Scenes", scene?.label ?? target.sceneId, entity.name || entity.id, componentLabel],
    selection: { kind: "component", scene, entity, component },
    canOpen: false,
    capabilities: [
      "component",
      "inspectable",
      "metadata-backed",
      component.assetRefs.length ? "has-asset-refs" : "no-asset-refs",
      component.properties.length ? "has-properties" : "raw-values-only",
      ...descriptorCapabilities,
    ],
    metadataTraits,
    metadataRefs: [
      metadataRef("targetKind", "component", "Component target"),
      metadataRef("component", component.typeName, component.label || component.typeName, "instance"),
      ...(component.descriptorKind ? [metadataRef("component", component.descriptorKind, component.descriptorKind, "descriptor-kind")] : []),
      ...metadataTraits.map((traitKind) =>
        metadataRef("metadataTrait", traitKind, traitKind, "component-trait"),
      ),
    ],
    documentRefs: scene?.documentPath
      ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, component.yamlPath, { kind: "scene", sceneId: target.sceneId })]
      : [],
    relatedTargets: [
      relatedRef("owner-entity", entity.name || entity.id, { kind: "sceneEntity", sceneId: target.sceneId, entityId: entity.id }, "owner entity"),
      ...(scene ? [relatedRef("scene", scene.label || scene.id, { kind: "scene", sceneId: scene.id }, "owner scene")] : []),
      ...assetTargets,
    ],
    diagnostics: [
      ...diagnosticRefs(component.diagnostics),
      ...diagnosticsForTarget(services, [scene?.documentPath], [entity.id, entity.name, component.typeName, component.yamlPath]),
    ],
    actions: [
      action("inspect", "Inspect component", "primary", "inspect"),
      action("owner", "Select owner entity", "default", "select"),
      action("reveal", "Reveal", "default", "reveal"),
    ],
  });
}

function findSceneOwnedComponent(
  _services: WorkspaceRuntimeServices,
  _target: Extract<EditorTargetRef, { kind: "component" }>,
): EditorSceneComponentInstanceDto | null {
  return null;
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
      documentRefs: scene?.documentPath
        ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
        : [],
      metadataRefs: [metadataRef("targetKind", "uiDocument", "UI document target")],
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
    capabilities: ["ui-document", "openable", "inspectable", "has-ui-tree"],
    metadataTraits: ["UiEditable", "HasUiTree", "GenericEditable"],
    metadataRefs: [
      metadataRef("targetKind", "uiDocument", "UI document target"),
      metadataRef("documentKind", "uiDocument", "UI document"),
    ],
    documentRefs: scene?.documentPath
      ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
      : [],
    relatedTargets: [
      relatedRef("owner-entity", document.entityName, { kind: "sceneEntity", sceneId: target.sceneId, entityId: document.entityId }),
      relatedRef("root-node", document.root.label || document.root.id, {
        kind: "uiNode",
        sceneId: target.sceneId,
        entityId: target.entityId,
        componentIndex: target.componentIndex,
        nodePath: document.root.path,
      }),
      ...document.bindings.slice(0, 8).map((binding) => relatedRef("binding", binding.id, undefined, binding.path)),
    ],
    diagnostics: diagnosticsForTarget(services, [scene?.documentPath], [document.entityId, document.entityName]),
    actions: [
      action("open", "Open UI document", "primary", "open"),
      action("reveal", "Reveal", "default", "reveal"),
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
      documentRefs: scene?.documentPath
        ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
        : [],
      metadataRefs: [metadataRef("targetKind", "uiNode", "UI node target")],
    });
  }

  return resolved({
    ref: target,
    label: node.label || node.id,
    subtitle: `${node.kind} - ${node.path}`,
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
    capabilities: [
      "ui-node",
      "inspectable",
      "selectable",
      node.visible ? "visible" : "hidden",
      node.enabled ? "enabled" : "disabled",
      node.actionEvent ? "has-action" : "",
      node.childCount ? "has-children" : "",
    ].filter(Boolean),
    metadataTraits: ["UiEditable", "HasUiTree", "GenericEditable"],
    metadataRefs: [
      metadataRef("targetKind", "uiNode", "UI node target"),
      metadataRef("uiNodeKind", node.kind, humanize(node.kind), "kind"),
    ],
    documentRefs: scene?.documentPath
      ? [documentRef("sceneYaml", "Scene YAML", scene.documentPath, "owner-scene", { kind: "scene", sceneId: target.sceneId })]
      : [],
    relatedTargets: [
      relatedRef("owner-document", `${document.entityName} UI`, {
        kind: "uiDocument",
        sceneId: target.sceneId,
        entityId: target.entityId,
        componentIndex: target.componentIndex,
      }),
      ...node.children.slice(0, 12).map((child) => relatedRef("child-node", child.label || child.id, {
        kind: "uiNode",
        sceneId: target.sceneId,
        entityId: target.entityId,
        componentIndex: target.componentIndex,
        nodePath: child.path,
      }, child.kind)),
    ],
    diagnostics: diagnosticsForTarget(services, [scene?.documentPath], [node.id, node.label, node.path]),
    actions: [
      action("open", "Open focused UI view", "primary", "open"),
      action("reveal", "Reveal", "default", "reveal"),
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
  const path = diagnostic?.path ?? target.path ?? null;

  const base = diagnostic
    ? resolved({
        ref: target,
        label,
        subtitle,
        icon: diagnostic.level === "error" ? "CircleX" : "TriangleAlert",
        breadcrumbs: ["Diagnostics", label],
        selection: emptyEditorTargetSelection(),
        canOpen: Boolean(path),
        capabilities: ["diagnostic", "revealable", "inspectable"],
        metadataRefs: [
          metadataRef("targetKind", "diagnostic", "Diagnostic target"),
          metadataRef("custom", diagnostic.level, diagnostic.level, "level"),
        ],
        documentRefs: path
          ? [documentRef("diagnosticSource", basename(path), path, "diagnostic-source", { kind: "projectFile", path }, true)]
          : [],
        relatedTargets: path
          ? [relatedRef("source", path, { kind: "projectFile", path })]
          : [],
        diagnostics: [diagnosticRef(diagnostic)],
        actions: path ? [action("reveal", "Reveal source", "primary", "reveal")] : [],
      })
    : missing({
        ref: target,
        label,
        subtitle,
        icon: "TriangleAlert",
        breadcrumbs: ["Diagnostics", label],
        reason: `Diagnostic not found: ${target.diagnosticId}`,
        documentRefs: path
          ? [documentRef("diagnosticSource", basename(path), path, "diagnostic-source", { kind: "projectFile", path }, true)]
          : [],
        metadataRefs: [metadataRef("targetKind", "diagnostic", "Diagnostic target")],
      });

  return base;
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
    capabilities: ["capability", "metadata", "inspectable"],
    metadataRefs: [
      metadataRef("targetKind", "capability", "Capability target"),
      metadataRef("capability", capabilityId, capabilityId),
    ],
    documentRefs: [],
    relatedTargets: [],
    diagnostics: [],
    actions: [
      action("open", "Open capabilities", "primary", "open"),
      action("inspect", "Inspect", "default", "inspect"),
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
    capabilities: ["dependency", "metadata", "inspectable"],
    metadataRefs: [
      metadataRef("targetKind", "dependency", "Dependency target"),
      metadataRef("dependency", dependencyId, dependencyId),
    ],
    documentRefs: [],
    relatedTargets: [],
    diagnostics: [],
    actions: [
      action("open", "Open dependencies", "primary", "open"),
      action("inspect", "Inspect", "default", "inspect"),
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
  capabilities = [],
  metadataTraits = [],
  metadataRefs = [],
  documentRefs = [],
  relatedTargets = [],
  diagnostics = [],
}: {
  ref: EditorTargetRef;
  label: string;
  subtitle?: string;
  icon: string;
  breadcrumbs: string[];
  selection: EditorSelection;
  canOpen: boolean;
  actions: EditorTargetAction[];
  capabilities?: string[];
  metadataTraits?: string[];
  metadataRefs?: EditorTargetMetadataRef[];
  documentRefs?: EditorTargetDocumentRef[];
  relatedTargets?: EditorTargetRelatedRef[];
  diagnostics?: EditorTargetDiagnosticRef[];
}): ResolvedEditorTarget {
  const descriptor: EditorTargetDescriptor = {
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
  };

  return {
    ref,
    status: "resolved",
    selection,
    descriptor,
    capabilities: uniqueStrings(capabilities),
    metadataTraits: uniqueStrings(metadataTraits),
    metadataRefs,
    documentRefs,
    relatedTargets,
    diagnostics,
    breadcrumbs: breadcrumbs.map((entry) => ({ label: entry })),
    actions,
  };
}

function missing({
  ref,
  label,
  subtitle,
  icon,
  breadcrumbs,
  reason,
  capabilities = [],
  metadataRefs = [],
  documentRefs = [],
  relatedTargets = [],
  diagnostics = [],
}: {
  ref: EditorTargetRef;
  label: string;
  subtitle?: string;
  icon: string;
  breadcrumbs: string[];
  reason: string;
  capabilities?: string[];
  metadataRefs?: EditorTargetMetadataRef[];
  documentRefs?: EditorTargetDocumentRef[];
  relatedTargets?: EditorTargetRelatedRef[];
  diagnostics?: EditorTargetDiagnosticRef[];
}): ResolvedEditorTarget {
  const descriptor: EditorTargetDescriptor = {
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
  };

  return {
    ref,
    status: "missing",
    reason,
    selection: emptyEditorTargetSelection(),
    descriptor,
    capabilities: uniqueStrings(["missing", ...capabilities]),
    metadataTraits: [],
    metadataRefs,
    documentRefs,
    relatedTargets,
    diagnostics,
    breadcrumbs: breadcrumbs.map((entry) => ({ label: entry })),
    actions: [],
  };
}

function action(
  id: string,
  label: string,
  tone: EditorTargetAction["tone"] = "default",
  intent?: EditorTargetIntent,
): EditorTargetAction {
  return {
    id,
    label,
    tone,
    intent,
    enabled: true,
    visible: true,
  };
}

function metadataRef(
  kind: EditorTargetMetadataRef["kind"],
  id: string,
  label = id,
  role?: string,
): EditorTargetMetadataRef {
  return {
    kind,
    id,
    label,
    role,
  };
}

function documentRef(
  kind: EditorTargetDocumentRef["kind"],
  label: string,
  path: string | null | undefined,
  role: string,
  target?: EditorTargetRef,
  readonly = false,
): EditorTargetDocumentRef {
  return {
    kind,
    label,
    path,
    role,
    target,
    readonly,
  };
}

function relatedRef(
  relation: string,
  label: string,
  target?: EditorTargetRef,
  detail?: string,
): EditorTargetRelatedRef {
  return {
    relation,
    label,
    target,
    detail,
  };
}

function diagnosticRef(diagnostic: EditorDiagnosticDto): EditorTargetDiagnosticRef {
  return {
    level: diagnostic.level,
    code: diagnostic.code,
    message: diagnostic.message,
    path: diagnostic.path ?? null,
  };
}

function diagnosticRefs(diagnostics: EditorDiagnosticDto[]): EditorTargetDiagnosticRef[] {
  return diagnostics.map(diagnosticRef);
}

function diagnosticsForTarget(
  services: WorkspaceRuntimeServices,
  paths: Array<string | null | undefined>,
  labels: Array<string | null | undefined>,
): EditorTargetDiagnosticRef[] {
  const normalizedPaths = paths
    .filter((path): path is string => Boolean(path))
    .map(normalizePath);
  const searchLabels = labels
    .filter((label): label is string => Boolean(label))
    .map((label) => label.toLowerCase());

  return (services.allProblems ?? [])
    .filter((diagnostic) => {
      const diagnosticPath = normalizePath(diagnostic.path ?? "");
      const diagnosticText = `${diagnostic.code} ${diagnostic.message}`.toLowerCase();

      return (
        normalizedPaths.some((path) => Boolean(path && diagnosticPath.includes(path))) ||
        searchLabels.some((label) => Boolean(label && diagnosticText.includes(label)))
      );
    })
    .map(diagnosticRef);
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

function findComponent(
  entity: EditorSceneEntityDto,
  componentIndex: number,
  componentType: string,
): EditorSceneComponentInstanceDto | null {
  return (
    entity.components?.find(
      (component) => component.componentIndex === componentIndex && component.typeName === componentType,
    ) ??
    entity.components?.find((component) => component.componentIndex === componentIndex) ??
    null
  );
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

function uniqueStrings(values: string[]): string[] {
  return Array.from(new Set(values.filter(Boolean)));
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
