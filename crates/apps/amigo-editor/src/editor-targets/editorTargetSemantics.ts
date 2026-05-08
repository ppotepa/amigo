// @codemap anchor:editor-target-semantics domain:workspace role:target-model priority:P1 layer:app tags:editor-target,semantics,cleanup
//
// Semantic target taxonomy for Amigo editor.
//
// This file intentionally evolves the current active editor-target system in place.
// It does not introduce a parallel target system and it does not create a "v2"
// namespace. Existing broad target kinds can continue to be used while call sites
// are migrated to more precise semantic roles.

import type {
  EditorTargetKind,
  EditorTargetRef,
} from "./editorTargetTypes";

export type EditorSemanticTargetKind =
  | "project.root"
  | "project.folder"
  | "project.file"
  | "mod.definition"
  | "asset.definition"
  | "asset.file"
  | "asset.usage"
  | "scene.document"
  | "scene.entity"
  | "scene.component"
  | "prefab.definition"
  | "prefab.instance"
  | "ui.document"
  | "ui.node"
  | "script.file"
  | "diagnostic"
  | "capability"
  | "dependency";

export type EditorTargetSemanticRole =
  | "navigation"
  | "document"
  | "domain-object"
  | "usage"
  | "runtime-scene-object"
  | "component"
  | "ui-node"
  | "metadata"
  | "diagnostic"
  | "dependency";

export type EditorTargetSemanticDescriptor = {
  semanticKind: EditorSemanticTargetKind;
  targetKind: EditorTargetKind;
  label: string;
  role: EditorTargetSemanticRole;
  description: string;
  preferredIntents: Array<"select" | "open" | "reveal" | "inspect" | "contextMenu">;
  capabilities: string[];
};

export const EDITOR_TARGET_SEMANTICS: Record<
  EditorSemanticTargetKind,
  EditorTargetSemanticDescriptor
> = {
  "project.root": {
    semanticKind: "project.root",
    targetKind: "projectNode",
    label: "Project Root",
    role: "navigation",
    description: "Root navigation target for the current project/mod workspace.",
    preferredIntents: ["select", "inspect", "open"],
    capabilities: ["navigable", "has-children"],
  },
  "project.folder": {
    semanticKind: "project.folder",
    targetKind: "projectNode",
    label: "Project Folder",
    role: "navigation",
    description: "Folder-like navigation item in the project tree.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["navigable", "has-children"],
  },
  "project.file": {
    semanticKind: "project.file",
    targetKind: "projectFile",
    label: "Project File",
    role: "document",
    description: "Physical source file in the project tree.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "revealable", "diagnostic-host"],
  },
  "mod.definition": {
    semanticKind: "mod.definition",
    targetKind: "mod",
    label: "Mod Definition",
    role: "domain-object",
    description: "Current mod/project definition and overview context.",
    preferredIntents: ["select", "inspect", "open"],
    capabilities: ["inspectable", "has-assets", "has-scenes"],
  },
  "asset.definition": {
    semanticKind: "asset.definition",
    targetKind: "asset",
    label: "Asset Definition",
    role: "domain-object",
    description: "Logical asset entry from asset metadata or an asset registry.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["previewable", "openable", "has-backing-file", "has-usages"],
  },
  "asset.file": {
    semanticKind: "asset.file",
    targetKind: "projectFile",
    label: "Asset File",
    role: "document",
    description: "Physical file that backs one or more logical asset definitions.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["previewable", "openable", "backing-file"],
  },
  "asset.usage": {
    semanticKind: "asset.usage",
    targetKind: "asset",
    label: "Asset Usage",
    role: "usage",
    description: "A reference to an asset from a scene, component, prefab, UI node, or document.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["usage", "revealable", "has-owner-target"],
  },
  "scene.document": {
    semanticKind: "scene.document",
    targetKind: "scene",
    label: "Scene Document",
    role: "document",
    description: "Scene document as an editable YAML/source document.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "previewable", "yaml-document", "has-entities"],
  },
  "scene.entity": {
    semanticKind: "scene.entity",
    targetKind: "sceneEntity",
    label: "Scene Entity",
    role: "runtime-scene-object",
    description: "Entity instance contained in a scene.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["selectable", "focusable", "has-components"],
  },
  "scene.component": {
    semanticKind: "scene.component",
    targetKind: "sceneEntity",
    label: "Scene Component",
    role: "component",
    description: "Component attached to a scene entity.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["inspectable", "has-properties", "patchable"],
  },
  "prefab.definition": {
    semanticKind: "prefab.definition",
    targetKind: "asset",
    label: "Prefab Definition",
    role: "domain-object",
    description: "Prefab asset definition.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "instantiable", "has-usages"],
  },
  "prefab.instance": {
    semanticKind: "prefab.instance",
    targetKind: "sceneEntity",
    label: "Prefab Instance",
    role: "runtime-scene-object",
    description: "Scene entity instance backed by a prefab definition.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["selectable", "focusable", "has-source-prefab"],
  },
  "ui.document": {
    semanticKind: "ui.document",
    targetKind: "uiDocument",
    label: "UI Document",
    role: "document",
    description: "UI document authored in the editor.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "has-ui-tree", "patchable"],
  },
  "ui.node": {
    semanticKind: "ui.node",
    targetKind: "uiNode",
    label: "UI Node",
    role: "ui-node",
    description: "Node inside a UI document tree.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["selectable", "inspectable", "has-properties"],
  },
  "script.file": {
    semanticKind: "script.file",
    targetKind: "script",
    label: "Script File",
    role: "document",
    description: "Script source file.",
    preferredIntents: ["select", "inspect", "open", "reveal"],
    capabilities: ["openable", "source-file", "diagnostic-host"],
  },
  diagnostic: {
    semanticKind: "diagnostic",
    targetKind: "diagnostic",
    label: "Diagnostic",
    role: "diagnostic",
    description: "Diagnostic or validation problem attached to a source/target.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["revealable", "has-owner-target"],
  },
  capability: {
    semanticKind: "capability",
    targetKind: "capability",
    label: "Capability",
    role: "metadata",
    description: "Capability metadata entry.",
    preferredIntents: ["select", "inspect"],
    capabilities: ["metadata", "inspectable"],
  },
  dependency: {
    semanticKind: "dependency",
    targetKind: "dependency",
    label: "Dependency",
    role: "dependency",
    description: "Dependency metadata entry.",
    preferredIntents: ["select", "inspect", "reveal"],
    capabilities: ["metadata", "inspectable", "has-usages"],
  },
};

export function semanticDescriptorForTarget(
  target: EditorTargetRef,
): EditorTargetSemanticDescriptor {
  return EDITOR_TARGET_SEMANTICS[semanticKindForTarget(target)];
}

export function semanticKindForTarget(
  target: EditorTargetRef,
): EditorSemanticTargetKind {
  switch (target.kind) {
    case "mod":
      return "mod.definition";

    case "projectNode":
      if (target.nodeKind === "modRoot" || target.nodeKind === "root") {
        return "project.root";
      }

      if (target.path || target.expectedPath) {
        return "project.file";
      }

      return "project.folder";

    case "projectFile":
      return isLikelyAssetPath(target.path) ? "asset.file" : "project.file";

    case "script":
      return "script.file";

    case "asset":
      return "asset.definition";

    case "scene":
      return "scene.document";

    case "sceneEntity":
      return "scene.entity";

    case "component":
      return "scene.component";

    case "uiDocument":
      return "ui.document";

    case "uiNode":
      return "ui.node";

    case "diagnostic":
      return "diagnostic";

    case "capability":
      return "capability";

    case "dependency":
      return "dependency";
  }
}

export function semanticLabelForTarget(target: EditorTargetRef): string {
  return semanticDescriptorForTarget(target).label;
}

export function semanticCapabilitiesForTarget(target: EditorTargetRef): string[] {
  return semanticDescriptorForTarget(target).capabilities;
}

export function semanticRoleForTarget(target: EditorTargetRef): EditorTargetSemanticRole {
  return semanticDescriptorForTarget(target).role;
}

function isLikelyAssetPath(path: string): boolean {
  const normalized = path.replace(/\\/g, "/").toLowerCase();

  return (
    normalized.includes("/assets/") ||
    normalized.endsWith(".png") ||
    normalized.endsWith(".jpg") ||
    normalized.endsWith(".jpeg") ||
    normalized.endsWith(".webp") ||
    normalized.endsWith(".svg") ||
    normalized.endsWith(".aseprite") ||
    normalized.endsWith(".wav") ||
    normalized.endsWith(".ogg") ||
    normalized.endsWith(".mp3") ||
    normalized.endsWith(".ttf") ||
    normalized.endsWith(".otf")
  );
}
