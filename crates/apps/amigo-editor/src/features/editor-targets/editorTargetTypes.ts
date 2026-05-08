// @codemap anchor:editor-target-types domain:editor role:target-model
// Central semantic model for editor targets.
//
// Surfaces such as Project Tree, Asset Tree, Scene Tree, UI Tree,
// Diagnostics, and Viewport should emit EditorTargetRef + intent.
// They must not directly decide right-dock content.

export type EditorTargetKind =
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

export type EditorTargetIntent =
  | "select"
  | "open"
  | "reveal"
  | "inspect"
  | "focus"
  | "contextMenu"
  | "showUsages";

export type EditorTargetSource =
  | "projectTree"
  | "assetTree"
  | "sceneTree"
  | "viewport"
  | "diagnostics"
  | "uiTree"
  | "search"
  | "breadcrumb"
  | "unknown";

export interface EditorTargetRef {
  kind: EditorTargetKind;
  id: string;
  label?: string;
  path?: string;
  parentId?: string;
  source?: EditorTargetSource;
  intent?: EditorTargetIntent;
  data?: Record<string, unknown>;
}

export type ResolvedEditorTargetStatus =
  | "resolved"
  | "missing"
  | "unsupported";

export interface EditorTargetDescriptor {
  kind: EditorTargetKind;
  label: string;
  description?: string;
  icon?: string;
  allowedIntents: EditorTargetIntent[];
  capabilities: string[];
  primaryPanels: string[];
  secondaryPanels: string[];
  defaultIntent: EditorTargetIntent;
}

export interface EditorTargetContextProfile {
  primary: string[];
  secondary: string[];
  defaultAction?: EditorTargetIntent;
}

export interface ResolvedEditorTarget {
  ref: EditorTargetRef;
  status: ResolvedEditorTargetStatus;
  descriptor: EditorTargetDescriptor;
  contextProfile: EditorTargetContextProfile;
  reason?: string;
  breadcrumbs?: EditorTargetRef[];
  relatedTargets?: EditorTargetRef[];
  diagnostics?: EditorTargetRef[];
  capabilities?: string[];
}

export function editorTargetKey(ref: EditorTargetRef): string {
  return `${ref.kind}:${ref.id}`;
}

export function withEditorTargetIntent(
  ref: EditorTargetRef,
  intent: EditorTargetIntent,
  source?: EditorTargetSource,
): EditorTargetRef {
  return {
    ...ref,
    intent,
    source: source ?? ref.source ?? "unknown",
  };
}

export function isAssetTarget(kind: EditorTargetKind): boolean {
  return kind === "asset.definition" || kind === "asset.file" || kind === "asset.usage";
}

export function isSceneTarget(kind: EditorTargetKind): boolean {
  return kind === "scene.document" || kind === "scene.entity" || kind === "scene.component";
}

export function isUiTarget(kind: EditorTargetKind): boolean {
  return kind === "ui.document" || kind === "ui.node";
}

export function isProjectTarget(kind: EditorTargetKind): boolean {
  return kind === "project.root" || kind === "project.folder" || kind === "project.file";
}