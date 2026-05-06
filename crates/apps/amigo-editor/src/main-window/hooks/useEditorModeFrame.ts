import { useCallback, useState } from "react";
import type {
  EditorFrameDto,
  EditorFrameResultDto,
  EditorModeSessionDto,
  EditorSceneSnapshotDto,
} from "../../api/dto";

export function useEditorModeFrame({
  selectSceneEntity,
  selectUiNode,
}: {
  selectSceneEntity: (entityId: string | null) => void;
  selectUiNode: (selection: {
    entityId: string;
    componentIndex: number;
    nodePath: string;
  } | null) => void;
}) {
  const [editorSnapshot, setEditorSnapshot] = useState<EditorSceneSnapshotDto | null>(null);
  const [editorSnapshotSceneId, setEditorSnapshotSceneId] = useState<string | null>(null);
  const [editorModeSession, setEditorModeSession] = useState<EditorModeSessionDto | null>(null);
  const [editorFrame, setEditorFrame] = useState<EditorFrameDto | null>(null);

  const applyEditorFrameResult = useCallback(
    (result: EditorFrameResultDto | null | undefined) => {
      if (!result) return;

      if (result.session) {
        setEditorModeSession(result.session);
      }

      if (result.snapshot) {
        setEditorSnapshot(result.snapshot);
        setEditorSnapshotSceneId(result.snapshot.sceneId);

        const selectedUiNode = result.snapshot.selection?.selectedUiNode ?? null;
        if (selectedUiNode) {
          selectUiNode({
            entityId: selectedUiNode.entityId,
            componentIndex: selectedUiNode.componentIndex,
            nodePath: selectedUiNode.nodePath,
          });
        } else {
          const selectedEntityId = result.snapshot.selection?.selectedEntityIds[0] ?? null;
          selectSceneEntity(selectedEntityId);
        }
      }

      if (result.frame) {
        setEditorFrame(result.frame);
      }
    },
    [selectSceneEntity, selectUiNode],
  );

  return {
    applyEditorFrameResult,
    editorFrame,
    editorModeSession,
    editorSnapshot,
    editorSnapshotSceneId,
    setEditorFrame,
    setEditorModeSession,
    setEditorSnapshot,
    setEditorSnapshotSceneId,
  };
}
