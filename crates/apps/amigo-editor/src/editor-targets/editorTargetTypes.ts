import type { EditorSelection } from "../properties/propertiesTypes";
import type { EditorTargetContextProfile } from "./editorTargetContextTypes";

export type EditorTargetIntent =
  | "select"
  | "open"
  | "reveal"
  | "contextMenu"
  | "inspect";

export type EditorTargetActionTone =
  | "default"
  | "primary"
  | "danger"
  | "warning"
  | "success"
  | "muted";

export type EditorTargetAction = {
  id: string;
  label: string;
  title?: string;
  tone?: EditorTargetActionTone;
  enabled?: boolean;
  visible?: boolean;
  intent?: EditorTargetIntent;
};

export type EditorTargetKind =
  | "mod"
  | "projectNode"
  | "projectFile"
  | "script"
  | "asset"
  | "scene"
  | "sceneEntity"
  | "component"
  | "uiDocument"
  | "uiNode"
  | "diagnostic"
  | "capability"
  | "dependency";

// @codemap anchor:editor-target-ref domain:workspace role:model priority:P1 layer:app tags:editor-target,selection,right-dock
export type EditorTargetRef =
  | {
      kind: "mod";
      modId: string;
    }
  | {
      kind: "projectNode";
      nodeId: string;
      nodeKind: string;
      label?: string;
      path?: string | null;
      expectedPath?: string | null;
    }
  | {
      kind: "projectFile";
      path: string;
    }
  | {
      kind: "script";
      path: string;
    }
  | {
      kind: "asset";
      assetKey: string;
    }
  | {
      kind: "scene";
      sceneId: string;
    }
  | {
      kind: "sceneEntity";
      sceneId: string;
      entityId: string;
    }
  | {
      // @codemap anchor:editor-component-target-ref domain:workspace role:model priority:P1 layer:app tags:editor-target,component,metadata
      kind: "component";
      sceneId: string;
      entityId: string;
      componentIndex: number;
      componentType: string;
    }
  | {
      kind: "uiDocument";
      sceneId: string;
      entityId: string;
      componentIndex: number;
    }
  | {
      kind: "uiNode";
      sceneId: string;
      entityId: string;
      componentIndex: number;
      nodePath: string;
    }
  | {
      kind: "diagnostic";
      diagnosticId: string;
      code?: string | null;
      path?: string | null;
    }
  | {
      kind: "capability";
      capabilityId: string;
    }
  | {
      kind: "dependency";
      dependencyId: string;
    };

export type EditorTargetDescriptor = {
  kind: EditorTargetKind;
  label: string;
  subtitle?: string;
  icon: string;
  breadcrumbs: string[];
  canOpen: boolean;
  canReveal: boolean;
  canInspect: boolean;
  selectionKind: EditorSelection["kind"] | "none";
  actions: EditorTargetAction[];
};

export type EditorTargetBreadcrumb = {
  label: string;
  target?: EditorTargetRef;
};

export type EditorTargetMetadataRefKind =
  | "targetKind"
  | "component"
  | "assetKind"
  | "documentKind"
  | "control"
  | "patchOp"
  | "capability"
  | "dependency"
  | "uiNodeKind"
  | "custom";

export type EditorTargetMetadataRef = {
  kind: EditorTargetMetadataRefKind;
  id: string;
  label?: string;
  role?: string;
};

export type EditorTargetDocumentKind =
  | "projectFile"
  | "directory"
  | "sceneYaml"
  | "sceneScript"
  | "assetDescriptor"
  | "assetSource"
  | "uiDocument"
  | "diagnosticSource"
  | "unknown";

export type EditorTargetDocumentRef = {
  kind: EditorTargetDocumentKind;
  label: string;
  path?: string | null;
  role: string;
  readonly?: boolean;
  target?: EditorTargetRef;
};

export type EditorTargetRelatedRef = {
  relation: string;
  label: string;
  detail?: string;
  target?: EditorTargetRef;
};

export type EditorTargetDiagnosticRef = {
  level: "info" | "warning" | "error";
  code: string;
  message: string;
  path?: string | null;
};

export type ResolvedEditorTargetStatus = "resolved" | "missing" | "unsupported";

// @codemap anchor:resolved-editor-target domain:workspace role:model priority:P1 layer:app tags:editor-target,right-dock,selection
export type ResolvedEditorTarget = {
  ref: EditorTargetRef;
  status: ResolvedEditorTargetStatus;
  descriptor: EditorTargetDescriptor;
  contextProfile: EditorTargetContextProfile;
  selection: EditorSelection;
  reason?: string;

  capabilities: string[];
  metadataRefs: EditorTargetMetadataRef[];
  documentRefs: EditorTargetDocumentRef[];
  relatedTargets: EditorTargetRelatedRef[];
  diagnostics: EditorTargetDiagnosticRef[];
  breadcrumbs: EditorTargetBreadcrumb[];
  actions: EditorTargetAction[];
};

export function editorTargetKey(target: EditorTargetRef): string {
  switch (target.kind) {
    case "mod":
      return `mod:${target.modId}`;
    case "projectNode":
      return `project-node:${target.nodeId}`;
    case "projectFile":
      return `project-file:${target.path}`;
    case "script":
      return `script:${target.path}`;
    case "asset":
      return `asset:${target.assetKey}`;
    case "scene":
      return `scene:${target.sceneId}`;
    case "sceneEntity":
      return `scene-entity:${target.sceneId}:${target.entityId}`;
    case "component":
      return `component:${target.sceneId}:${target.entityId}:${target.componentIndex}:${target.componentType}`;
    case "uiDocument":
      return `ui-document:${target.sceneId}:${target.entityId}:${target.componentIndex}`;
    case "uiNode":
      return `ui-node:${target.sceneId}:${target.entityId}:${target.componentIndex}:${target.nodePath}`;
    case "diagnostic":
      return `diagnostic:${target.diagnosticId}`;
    case "capability":
      return `capability:${target.capabilityId}`;
    case "dependency":
      return `dependency:${target.dependencyId}`;
  }
}

export function emptyEditorTargetSelection(): EditorSelection {
  return { kind: "empty" };
}
