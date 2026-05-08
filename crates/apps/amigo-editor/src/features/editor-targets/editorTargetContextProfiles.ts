import type {
  EditorTargetContextProfile,
  EditorTargetDescriptor,
  EditorTargetKind,
} from "./editorTargetTypes";

// @codemap anchor:editor-target-context-profiles domain:editor role:target-context
// Stable right-dock context profile registry.
//
// right-top should render profile.primary.
// right-bottom should render profile.secondary.
//
// Left-side trees and navigation surfaces must not mutate dock layout.

const commonPrimary = ["target.header", "target.summary", "target.primaryActions"];
const commonSecondary = ["target.related", "target.diagnostics", "target.history"];

export const editorTargetDescriptors: Record<EditorTargetKind, EditorTargetDescriptor> = {
  "project.root": {
    kind: "project.root",
    label: "Project",
    icon: "folderRoot",
    allowedIntents: ["select", "inspect", "open"],
    capabilities: ["navigable", "hasChildren"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.children", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "project.folder": {
    kind: "project.folder",
    label: "Folder",
    icon: "folder",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["navigable", "hasChildren"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.children", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "project.file": {
    kind: "project.file",
    label: "File",
    icon: "file",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "revealable", "diagnosticHost"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.fileDetails", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "mod.definition": {
    kind: "mod.definition",
    label: "Mod",
    icon: "package",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "hasDocuments", "hasAssets"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.documents", "target.assets", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "asset.definition": {
    kind: "asset.definition",
    label: "Asset",
    icon: "image",
    allowedIntents: ["select", "inspect", "open", "reveal", "showUsages"],
    capabilities: ["previewable", "openable", "hasUsages", "hasBackingFile"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.preview", "target.usages", "target.backingFile", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "asset.file": {
    kind: "asset.file",
    label: "Asset File",
    icon: "fileImage",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "previewable", "backingFile"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.preview", "target.assetDefinitions", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "asset.usage": {
    kind: "asset.usage",
    label: "Asset Usage",
    icon: "link",
    allowedIntents: ["select", "inspect", "reveal", "showUsages"],
    capabilities: ["revealable", "usage", "hasOwnerTarget"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.owner", "target.assetDefinition", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "scene.document": {
    kind: "scene.document",
    label: "Scene",
    icon: "map",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "previewable", "yamlDocument", "hasEntities"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.sceneEntities", "target.document", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "scene.entity": {
    kind: "scene.entity",
    label: "Scene Entity",
    icon: "box",
    allowedIntents: ["select", "inspect", "focus", "reveal"],
    capabilities: ["selectable", "focusable", "hasComponents"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.components", "target.controls", "target.patchOps", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "scene.component": {
    kind: "scene.component",
    label: "Scene Component",
    icon: "component",
    allowedIntents: ["select", "inspect", "reveal"],
    capabilities: ["inspectable", "hasProperties", "patchable"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.properties", "target.controls", "target.patchOps", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "prefab.definition": {
    kind: "prefab.definition",
    label: "Prefab",
    icon: "boxes",
    allowedIntents: ["select", "inspect", "open", "reveal", "showUsages"],
    capabilities: ["openable", "hasUsages", "instantiable"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.usages", "target.instances", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "prefab.instance": {
    kind: "prefab.instance",
    label: "Prefab Instance",
    icon: "box",
    allowedIntents: ["select", "inspect", "focus", "reveal"],
    capabilities: ["selectable", "focusable", "hasSourcePrefab"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.prefabDefinition", "target.overrides", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "ui.document": {
    kind: "ui.document",
    label: "UI Document",
    icon: "panelTop",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "yamlDocument", "hasUiNodes"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.uiTree", "target.document", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "ui.node": {
    kind: "ui.node",
    label: "UI Node",
    icon: "squareMousePointer",
    allowedIntents: ["select", "inspect", "focus", "reveal"],
    capabilities: ["selectable", "inspectable", "hasProperties"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.properties", "target.children", "target.bindings", ...commonSecondary],
    defaultIntent: "inspect",
  },
  "script.file": {
    kind: "script.file",
    label: "Script",
    icon: "scrollText",
    allowedIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "sourceFile", "diagnosticHost"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.fileDetails", "target.usages", ...commonSecondary],
    defaultIntent: "inspect",
  },
  diagnostic: {
    kind: "diagnostic",
    label: "Diagnostic",
    icon: "triangleAlert",
    allowedIntents: ["select", "inspect", "reveal"],
    capabilities: ["revealable", "hasOwnerTarget"],
    primaryPanels: ["target.header", "target.diagnosticDetails", "target.primaryActions"],
    secondaryPanels: ["target.owner", "target.related"],
    defaultIntent: "inspect",
  },
  capability: {
    kind: "capability",
    label: "Capability",
    icon: "badgeCheck",
    allowedIntents: ["select", "inspect", "showUsages"],
    capabilities: ["inspectable", "metadata"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.usages", "target.related"],
    defaultIntent: "inspect",
  },
  dependency: {
    kind: "dependency",
    label: "Dependency",
    icon: "gitBranch",
    allowedIntents: ["select", "inspect", "reveal", "showUsages"],
    capabilities: ["inspectable", "hasUsages", "revealable"],
    primaryPanels: commonPrimary,
    secondaryPanels: ["target.usages", "target.related"],
    defaultIntent: "inspect",
  },
};

export function getEditorTargetDescriptor(kind: EditorTargetKind): EditorTargetDescriptor {
  return editorTargetDescriptors[kind];
}

export function getEditorTargetContextProfile(kind: EditorTargetKind): EditorTargetContextProfile {
  const descriptor = getEditorTargetDescriptor(kind);

  return {
    primary: descriptor.primaryPanels,
    secondary: descriptor.secondaryPanels,
    defaultAction: descriptor.defaultIntent,
  };
}