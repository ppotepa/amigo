import type { EditorUiDocumentDto, EditorUiNodeDto } from "../../api/dto";
import type { EditorTargetRef } from "../editorTargetTypes";

// @codemap anchor:ui-document-target-adapter domain:ui-document role:tree-adapter priority:P1 layer:app tags:editor-target,ui-document,selection
export function uiDocumentToTarget({
  sceneId,
  document,
}: {
  sceneId: string;
  document: EditorUiDocumentDto;
}): EditorTargetRef {
  return {
    kind: "uiDocument",
    sceneId,
    entityId: document.entityId,
    componentIndex: document.componentIndex,
  };
}

export function uiNodeToTarget({
  sceneId,
  entityId,
  componentIndex,
  nodePath,
}: {
  sceneId: string;
  entityId: string;
  componentIndex: number;
  nodePath: string;
}): EditorTargetRef {
  return {
    kind: "uiNode",
    sceneId,
    entityId,
    componentIndex,
    nodePath,
  };
}

export function uiNodeDtoToTarget({
  sceneId,
  document,
  node,
}: {
  sceneId: string;
  document: EditorUiDocumentDto;
  node: EditorUiNodeDto;
}): EditorTargetRef {
  return uiNodeToTarget({
    sceneId,
    entityId: document.entityId,
    componentIndex: document.componentIndex,
    nodePath: node.path,
  });
}
