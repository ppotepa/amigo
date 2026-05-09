import type { IconKey } from "../editor-components/componentTypes";
import type { EditorTargetRef, ResolvedEditorTarget } from "./editorTargetTypes";

const FALLBACK_KIND_LABELS: Record<EditorTargetRef["kind"], string> = {
  mod: "Mod",
  projectNode: "Project Node",
  projectFile: "File",
  script: "Script",
  asset: "Asset",
  scene: "Scene",
  sceneEntity: "Entity",
  component: "Component",
  uiDocument: "UI Document",
  uiNode: "UI Node",
  diagnostic: "Diagnostic",
  capability: "Capability",
  dependency: "Dependency",
};

const TARGET_ICON_TO_COMPONENT_ICON: Record<string, IconKey> = {
  FolderTree: "folder",
  Package: "package",
  PackageX: "package",
  TriangleAlert: "alert-triangle",
  CircleX: "alert-triangle",
  Network: "grid",
  Box: "box",
  Boxes: "box",
  Puzzle: "box",
  Code2: "terminal",
  LayoutPanelTop: "layout-template",
  MousePointerClick: "plus",
  FileText: "list",
  Monitor: "gauge",
  MonitorX: "gauge",
  Plug: "settings",
};

export function contextKindLabelForTarget(target: ResolvedEditorTarget | null): string {
  if (!target) return "Context";
  const refLabel = firstMetadataRefLabel(target);
  if (refLabel) return refLabel;
  if (target.ref.kind === "asset" && target.selection.kind === "asset") {
    return target.selection.asset.kind || "Asset";
  }
  if (target.ref.kind === "component") {
    return target.ref.componentType || "Component";
  }
  return FALLBACK_KIND_LABELS[target.ref.kind] ?? target.descriptor.kind ?? "Context";
}

export function contextIconForTarget(target: ResolvedEditorTarget | null): IconKey {
  if (!target) return "box";
  return TARGET_ICON_TO_COMPONENT_ICON[target.descriptor.icon] ?? "box";
}

function firstMetadataRefLabel(target: ResolvedEditorTarget): string | null {
  const priorities = [
    "component",
    "assetKind",
    "documentKind",
    "uiNodeKind",
    "capability",
    "dependency",
    "targetKind",
  ] as const;
  for (const kind of priorities) {
    const ref = target.metadataRefs.find((entry) => entry.kind === kind && entry.label);
    if (ref?.label) return ref.label;
  }
  return null;
}
