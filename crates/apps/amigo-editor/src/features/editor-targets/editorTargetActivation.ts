import {
  type EditorTargetIntent,
  type EditorTargetRef,
  type EditorTargetSource,
  withEditorTargetIntent,
} from "./editorTargetTypes";
import { resolveEditorTarget } from "./editorTargetResolver";

// @codemap anchor:editor-target-activation domain:editor role:target-activation
// Shared activation helper for trees, viewport, diagnostics, breadcrumbs,
// and search results.
//
// Surfaces should call this helper or an equivalent app-state action rather
// than directly changing right-dock panel contents.

export interface EditorTargetActivationResult {
  target: EditorTargetRef;
  resolved: ReturnType<typeof resolveEditorTarget>;
}

export function activateEditorTarget(
  ref: EditorTargetRef,
  intent: EditorTargetIntent = ref.intent ?? "select",
  source: EditorTargetSource = ref.source ?? "unknown",
): EditorTargetActivationResult {
  const target = withEditorTargetIntent(ref, intent, source);
  const resolved = resolveEditorTarget({ ref: target });

  return {
    target,
    resolved,
  };
}

export function createProjectFileTarget(
  path: string,
  label?: string,
  source: EditorTargetSource = "projectTree",
): EditorTargetRef {
  return {
    kind: "project.file",
    id: path,
    path,
    label: label ?? path,
    source,
    intent: "inspect",
  };
}

export function createAssetDefinitionTarget(
  id: string,
  label?: string,
  source: EditorTargetSource = "assetTree",
): EditorTargetRef {
  return {
    kind: "asset.definition",
    id,
    label: label ?? id,
    source,
    intent: "inspect",
  };
}

export function createAssetFileTarget(
  path: string,
  label?: string,
  source: EditorTargetSource = "projectTree",
): EditorTargetRef {
  return {
    kind: "asset.file",
    id: path,
    path,
    label: label ?? path,
    source,
    intent: "inspect",
  };
}

export function createAssetUsageTarget(
  id: string,
  ownerTargetId: string,
  assetId: string,
  label?: string,
  source: EditorTargetSource = "sceneTree",
): EditorTargetRef {
  return {
    kind: "asset.usage",
    id,
    label: label ?? assetId,
    source,
    intent: "inspect",
    data: {
      ownerTargetId,
      assetId,
    },
  };
}

export function createSceneDocumentTarget(
  id: string,
  path?: string,
  label?: string,
  source: EditorTargetSource = "projectTree",
): EditorTargetRef {
  return {
    kind: "scene.document",
    id,
    path,
    label: label ?? id,
    source,
    intent: "inspect",
  };
}

export function createSceneEntityTarget(
  sceneId: string,
  entityId: string,
  label?: string,
  source: EditorTargetSource = "sceneTree",
): EditorTargetRef {
  return {
    kind: "scene.entity",
    id: `${sceneId}:${entityId}`,
    label: label ?? entityId,
    source,
    intent: "inspect",
    data: {
      sceneId,
      entityId,
    },
  };
}

export function createSceneComponentTarget(
  sceneId: string,
  entityId: string,
  componentId: string,
  label?: string,
  source: EditorTargetSource = "sceneTree",
): EditorTargetRef {
  return {
    kind: "scene.component",
    id: `${sceneId}:${entityId}:${componentId}`,
    label: label ?? componentId,
    parentId: `${sceneId}:${entityId}`,
    source,
    intent: "inspect",
    data: {
      sceneId,
      entityId,
      componentId,
    },
  };
}

export function createUiDocumentTarget(
  id: string,
  path?: string,
  label?: string,
  source: EditorTargetSource = "uiTree",
): EditorTargetRef {
  return {
    kind: "ui.document",
    id,
    path,
    label: label ?? id,
    source,
    intent: "inspect",
  };
}

export function createUiNodeTarget(
  documentId: string,
  nodeId: string,
  label?: string,
  source: EditorTargetSource = "uiTree",
): EditorTargetRef {
  return {
    kind: "ui.node",
    id: `${documentId}:${nodeId}`,
    label: label ?? nodeId,
    parentId: documentId,
    source,
    intent: "inspect",
    data: {
      documentId,
      nodeId,
    },
  };
}

export function createDiagnosticTarget(
  id: string,
  label?: string,
  owner?: EditorTargetRef,
  source: EditorTargetSource = "diagnostics",
): EditorTargetRef {
  return {
    kind: "diagnostic",
    id,
    label: label ?? id,
    source,
    intent: "inspect",
    data: {
      owner,
    },
  };
}