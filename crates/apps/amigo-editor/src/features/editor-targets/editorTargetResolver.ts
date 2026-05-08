import {
  editorTargetKey,
  type EditorTargetRef,
  type ResolvedEditorTarget,
} from "./editorTargetTypes";
import {
  getEditorTargetContextProfile,
  getEditorTargetDescriptor,
} from "./editorTargetContextProfiles";

// @codemap anchor:editor-target-resolver domain:editor role:target-resolution
// Central target resolver.
//
// This is the only layer that should translate a lightweight EditorTargetRef
// into the context consumed by right dock, breadcrumbs, actions, diagnostics,
// and future metadata-driven controls.

export interface EditorTargetResolverInput {
  ref: EditorTargetRef | null | undefined;
}

export function resolveEditorTarget(input: EditorTargetResolverInput): ResolvedEditorTarget | null {
  const ref = input.ref;

  if (!ref) {
    return null;
  }

  const descriptor = getEditorTargetDescriptor(ref.kind);

  if (!descriptor) {
    return {
      ref,
      status: "unsupported",
      descriptor: {
        kind: ref.kind,
        label: ref.label ?? ref.kind,
        allowedIntents: ["inspect"],
        capabilities: [],
        primaryPanels: ["target.header", "target.summary"],
        secondaryPanels: ["target.related"],
        defaultIntent: "inspect",
      },
      contextProfile: {
        primary: ["target.header", "target.summary"],
        secondary: ["target.related"],
        defaultAction: "inspect",
      },
      reason: `Unsupported editor target kind: ${ref.kind}`,
      breadcrumbs: [],
      relatedTargets: [],
      diagnostics: [],
      capabilities: [],
    };
  }

  const contextProfile = getEditorTargetContextProfile(ref.kind);

  return {
    ref,
    status: "resolved",
    descriptor,
    contextProfile,
    breadcrumbs: buildDefaultBreadcrumbs(ref),
    relatedTargets: buildDefaultRelatedTargets(ref),
    diagnostics: [],
    capabilities: descriptor.capabilities,
  };
}

function buildDefaultBreadcrumbs(ref: EditorTargetRef): EditorTargetRef[] {
  const breadcrumbs: EditorTargetRef[] = [];

  if (ref.parentId) {
    breadcrumbs.push({
      kind: "project.root",
      id: "project",
      label: "Project",
      intent: "inspect",
      source: ref.source,
    });
  }

  breadcrumbs.push(ref);
  return breadcrumbs;
}

function buildDefaultRelatedTargets(ref: EditorTargetRef): EditorTargetRef[] {
  const related: EditorTargetRef[] = [];

  if (ref.path && ref.kind !== "project.file") {
    related.push({
      kind: "project.file",
      id: ref.path,
      label: ref.path,
      path: ref.path,
      source: ref.source,
      intent: "reveal",
      data: {
        owner: editorTargetKey(ref),
      },
    });
  }

  return related;
}