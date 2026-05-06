import type {
  EditorSceneHierarchyDto,
  EditorUiDocumentDto,
} from "../../api/dto";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export type UiDocumentEditorTarget = {
  modId: string;
  sceneId: string;
  entityId: string;
  componentIndex: number;
};

export type UiDocumentTargetResolution =
  | {
      kind: "resolved";
      source: "component-context" | "ui-node-selection" | "entity-selection" | "single-scene-document";
      target: UiDocumentEditorTarget;
      document: EditorUiDocumentDto | null;
    }
  | {
      kind: "ambiguous";
      source: "multiple-scene-documents";
      documents: EditorUiDocumentDto[];
    }
  | {
      kind: "missing";
      source: "no-scene" | "no-ui-document";
      documents: EditorUiDocumentDto[];
    };

export function resolveUiDocumentEditorTarget({
  hierarchy,
  instance,
  services,
}: {
  hierarchy: EditorSceneHierarchyDto | null | undefined;
  instance: EditorComponentInstance;
  services: WorkspaceRuntimeServices;
}): UiDocumentTargetResolution {
  const documents = hierarchy?.uiDocuments ?? [];
  const modId = services.details?.id ?? "";
  const sceneId = services.selectedScene?.id ?? hierarchy?.sceneId ?? "";

  const contextTarget = targetFromComponentContext(instance.context, modId);
  if (contextTarget) {
    return {
      kind: "resolved",
      source: "component-context",
      target: contextTarget,
      document: findDocument(documents, contextTarget),
    };
  }

  const uiNodeSelectionTarget = targetFromUiNodeSelection(services.selection, modId, sceneId);
  if (uiNodeSelectionTarget) {
    return {
      kind: "resolved",
      source: "ui-node-selection",
      target: uiNodeSelectionTarget,
      document: findDocument(documents, uiNodeSelectionTarget),
    };
  }

  const entitySelectionTarget = targetFromSelectedEntity(services, documents, modId, sceneId);
  if (entitySelectionTarget) {
    return {
      kind: "resolved",
      source: "entity-selection",
      target: entitySelectionTarget,
      document: findDocument(documents, entitySelectionTarget),
    };
  }

  if (!sceneId) {
    return { kind: "missing", source: "no-scene", documents };
  }

  if (documents.length === 1) {
    const document = documents[0];
    const target = {
      modId,
      sceneId,
      entityId: document.entityId,
      componentIndex: document.componentIndex,
    };

    return {
      kind: "resolved",
      source: "single-scene-document",
      target,
      document,
    };
  }

  if (documents.length > 1) {
    return {
      kind: "ambiguous",
      source: "multiple-scene-documents",
      documents,
    };
  }

  return { kind: "missing", source: "no-ui-document", documents };
}

function targetFromComponentContext(
  context: EditorComponentInstance["context"] | undefined,
  modId: string,
): UiDocumentEditorTarget | null {
  if (!context?.sceneId || !context.entityId || context.componentIndex == null) {
    return null;
  }

  return {
    modId,
    sceneId: String(context.sceneId),
    entityId: String(context.entityId),
    componentIndex: Number(context.componentIndex),
  };
}

function targetFromUiNodeSelection(
  selection: WorkspaceRuntimeServices["selection"],
  modId: string,
  sceneId: string,
): UiDocumentEditorTarget | null {
  if (selection?.kind !== "uiNode") {
    return null;
  }

  return {
    modId,
    sceneId,
    entityId: selection.nodeRef.entityId,
    componentIndex: selection.nodeRef.componentIndex,
  };
}

function targetFromSelectedEntity(
  services: WorkspaceRuntimeServices,
  documents: EditorUiDocumentDto[],
  modId: string,
  sceneId: string,
): UiDocumentEditorTarget | null {
  const selectedEntityId =
    services.selection?.kind === "entity"
      ? services.selection.entity.id
      : services.selectedEntity?.id ?? null;

  if (!selectedEntityId) {
    return null;
  }

  const document = documents.find((candidate) => candidate.entityId === selectedEntityId);
  if (!document) {
    return null;
  }

  return {
    modId,
    sceneId,
    entityId: document.entityId,
    componentIndex: document.componentIndex,
  };
}

function findDocument(
  documents: EditorUiDocumentDto[],
  target: UiDocumentEditorTarget,
): EditorUiDocumentDto | null {
  return (
    documents.find(
      (candidate) =>
        candidate.entityId === target.entityId &&
        candidate.componentIndex === target.componentIndex,
    ) ?? null
  );
}
