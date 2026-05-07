import { useCallback } from "react";
import type { Dispatch } from "react";
import type { EditorEvent } from "../../editorEvents";
import { selectedModId, selectedSceneId } from "../../selectionSelectors";
import type { EditorSelectionRef, EditorUiNodeSelectionRef } from "../../selectionTypes";
import type { Action } from "../editorActions";

export function useSelectionActions({
  dispatch,
  emit,
  selectionForWorkspace,
}: {
  dispatch: Dispatch<Action>;
  emit: (event: EditorEvent) => void;
  selectionForWorkspace: (workspaceId?: string) => EditorSelectionRef;
}) {
  const selectSceneEntity = useCallback(
    (entityId: string | null, workspaceId = "main") => {
      const selection = selectionForWorkspace(workspaceId);
      const modId = selectedModId(selection);
      const sceneId = selectedSceneId(selection);
      if (!modId || !sceneId) return;

      if (!entityId) {
        dispatch({ type: "workspaceSelectionChanged", workspaceId, selection: { kind: "scene", modId, sceneId } });
        emit({ type: "InspectorContextChanged", contextKind: "scene", id: sceneId });
        return;
      }

      dispatch({ type: "workspaceSelectionChanged", workspaceId, selection: { kind: "entity", modId, sceneId, entityId } });
      emit({ type: "InspectorContextChanged", contextKind: "entity", id: entityId });
    },
    [dispatch, emit, selectionForWorkspace],
  );

  const selectUiNode = useCallback(
    (uiNode: Omit<EditorUiNodeSelectionRef, "kind" | "modId" | "sceneId"> | null, workspaceId = "main") => {
      const selection = selectionForWorkspace(workspaceId);
      const modId = selectedModId(selection);
      const sceneId = selectedSceneId(selection);
      if (!modId || !sceneId) return;

      if (!uiNode) {
        dispatch({ type: "workspaceSelectionChanged", workspaceId, selection: { kind: "scene", modId, sceneId } });
        emit({ type: "InspectorContextChanged", contextKind: "scene", id: sceneId });
        return;
      }

      dispatch({
        type: "workspaceSelectionChanged",
        workspaceId,
        selection: {
          kind: "uiNode",
          modId,
          sceneId,
          entityId: uiNode.entityId,
          componentIndex: uiNode.componentIndex,
          nodePath: uiNode.nodePath,
        },
      });
      emit({
        type: "InspectorContextChanged",
        contextKind: "uiNode",
        id: `${uiNode.entityId}:${uiNode.componentIndex}:${uiNode.nodePath}`,
      });
    },
    [dispatch, emit, selectionForWorkspace],
  );

  return {
    selectSceneEntity,
    selectUiNode,
  };
}
