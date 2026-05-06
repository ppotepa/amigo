import type { EditorSceneHierarchyDto, EditorUiDocumentDto } from "../../api/dto";
import type { EditorComponentInstance } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export type UiDocumentEditorTarget = {
  modId: string;
  sceneId: string;
  entityId: string;
  componentIndex: number;
};

export type UiDocumentEditorTargetResolution =
  | {
      kind: "resolved";
      target: UiDocumentEditorTarget;
      document: EditorUiDocumentDto;
    }
  | {
      kind: "emptyScene";
      modId: string;
      sceneId: string;
      preferredEntityId?: string | null;
      initialTemplate?: string | null;
    }
  | {
      kind: "multipleDocuments";
      modId: string;
      sceneId: string;
      documents: EditorUiDocumentDto[];
    }
  | {
      kind: "noScene";
      message: string;
    }
  | {
      kind: "missingTarget";
      modId: string;
      sceneId: string;
      entityId: string;
      componentIndex?: number | null;
      preferredEntityId?: string | null;
      initialTemplate?: string | null;
    };

export function resolveUiDocumentEditorTarget({
  hierarchy,
  instance,
  services,
}: {
  hierarchy: EditorSceneHierarchyDto | null | undefined;
  instance: EditorComponentInstance;
  services: WorkspaceRuntimeServices;
}): UiDocumentEditorTargetResolution {
  const context = instance.context ?? {};
  const documents = hierarchy?.uiDocuments ?? [];
  const modId = services.details?.id ?? "";
  const sceneId = String(context.sceneId ?? services.selectedScene?.id ?? sceneIdFromSelection(services.selection) ?? hierarchy?.sceneId ?? "");

  if (!sceneId) {
    return {
      kind: "noScene",
      message: "Select a scene before creating or editing a UI document.",
    };
  }

  const preferredEntityId = stringOrNull(context.preferredEntityId ?? context.entityId);
  const initialTemplate = stringOrNull(context.initialTemplate);
  const contextEntityId = stringOrNull(context.entityId);
  const contextComponentIndex = context.componentIndex == null ? null : Number(context.componentIndex);

  if (contextEntityId && contextComponentIndex != null) {
    const document =
      documents.find(
        (candidate) =>
          candidate.entityId === contextEntityId &&
          candidate.componentIndex === contextComponentIndex,
      ) ?? null;

    if (document) {
      return {
        kind: "resolved",
        target: {
          modId,
          sceneId,
          entityId: document.entityId,
          componentIndex: document.componentIndex,
        },
        document,
      };
    }

    return {
      kind: "missingTarget",
      modId,
      sceneId,
      entityId: contextEntityId,
      componentIndex: contextComponentIndex,
      preferredEntityId,
      initialTemplate,
    };
  }

  const selection = services.selection;
  if (selection?.kind === "uiNode" && (!selection.scene || selection.scene.id === sceneId)) {
    const selectedDocument =
      documents.find(
        (candidate) =>
          candidate.entityId === selection.nodeRef.entityId &&
          candidate.componentIndex === selection.nodeRef.componentIndex,
      ) ?? null;

    if (selectedDocument) {
      return {
        kind: "resolved",
        target: {
          modId,
          sceneId,
          entityId: selectedDocument.entityId,
          componentIndex: selectedDocument.componentIndex,
        },
        document: selectedDocument,
      };
    }
  }

  const selectedEntityId =
    services.selection?.kind === "entity"
      ? services.selection.entity.id
      : services.selectedEntity?.id ?? null;
  if (selectedEntityId) {
    const selectedEntityDocument = documents.find((candidate) => candidate.entityId === selectedEntityId);
    if (selectedEntityDocument) {
      return {
        kind: "resolved",
        target: {
          modId,
          sceneId,
          entityId: selectedEntityDocument.entityId,
          componentIndex: selectedEntityDocument.componentIndex,
        },
        document: selectedEntityDocument,
      };
    }
  }

  if (documents.length === 1) {
    const document = documents[0];
    return {
      kind: "resolved",
      target: {
        modId,
        sceneId,
        entityId: document.entityId,
        componentIndex: document.componentIndex,
      },
      document,
    };
  }

  if (documents.length > 1) {
    return {
      kind: "multipleDocuments",
      modId,
      sceneId,
      documents,
    };
  }

  return {
    kind: "emptyScene",
    modId,
    sceneId,
    preferredEntityId,
    initialTemplate,
  };
}

function sceneIdFromSelection(selection: WorkspaceRuntimeServices["selection"]): string | null {
  if (selection?.kind === "scene") {
    return selection.scene.id;
  }
  if (selection?.kind === "entity" || selection?.kind === "uiNode") {
    return selection.scene?.id ?? null;
  }
  return null;
}

function stringOrNull(value: unknown): string | null {
  if (typeof value !== "string") {
    return null;
  }
  const trimmed = value.trim();
  return trimmed.length ? trimmed : null;
}
