import { useCallback } from "react";
import type { MutableRefObject } from "react";
import {
  applyEditorCommand as applyEditorCommandApi,
  applyEditorModeCommand as applyEditorModeCommandApi,
} from "../../api/editorApi";
import type {
  EditorCommandDto,
  EditorCommandResultDto,
  EditorFrameResultDto,
  EditorSceneSummaryDto,
  EditorSceneSnapshotDto,
} from "../../api/dto";
import type { EditorEvent } from "../../app/editorEvents";
import {
  sceneEditorPreviewFailed,
  sceneEditorPreviewReady,
  sceneEditorPreviewRegenerating,
  type SceneEditorPreviewSyncState,
} from "../../features/scenes/editor/sceneEditorPreviewSync";

function commandChangesSceneDocument(command: EditorCommandDto): boolean {
  return (
    command.type === "SetEntityTransform2D" ||
    command.type === "SetTileMapMarker2D" ||
    command.type === "SetAttachedLocalOffset2D" ||
    command.type === "SetUiNodeProperty" ||
    command.type === "CreateUiDocument" ||
    command.type === "AddUiNode" ||
    command.type === "AddUiTemplate" ||
    command.type === "DuplicateUiNode" ||
    command.type === "RemoveUiNode" ||
    command.type === "MoveUiNode"
  );
}

function commandChangesUiStructure(command: EditorCommandDto): boolean {
  return (
    command.type === "SetUiNodeProperty" ||
    command.type === "CreateUiDocument" ||
    command.type === "AddUiNode" ||
    command.type === "AddUiTemplate" ||
    command.type === "DuplicateUiNode" ||
    command.type === "RemoveUiNode" ||
    command.type === "MoveUiNode"
  );
}

function changedEntitiesForCommand(command: EditorCommandDto): string[] {
  switch (command.type) {
    case "SetEntityTransform2D":
    case "MoveEntity2D":
    case "SetTileMapMarker2D":
    case "SetAttachedLocalOffset2D":
    case "SetUiNodeProperty":
    case "CreateUiDocument":
    case "AddUiNode":
    case "AddUiTemplate":
    case "DuplicateUiNode":
    case "RemoveUiNode":
    case "MoveUiNode":
      return [command.entityId];
    default:
      return [];
  }
}

export function useApplyEditorCommand({
  applyEditorFrameResult,
  editorModeSessionId,
  loadEditorModeSceneHierarchy,
  loadSceneHierarchy,
  modId,
  previewSyncRevisionRef,
  recordEvent,
  regeneratePreview,
  refreshEditorSnapshotForScene,
  selectedScene,
  sessionId,
  setEditorPreviewSync,
  setEditorSnapshot,
  setEditorSnapshotSceneId,
}: {
  applyEditorFrameResult: (result: EditorFrameResultDto | null | undefined) => void;
  editorModeSessionId?: string | null;
  loadEditorModeSceneHierarchy: (sessionId: string, editorModeSessionId: string) => Promise<void>;
  loadSceneHierarchy: (modId: string, sceneId: string, force?: boolean) => Promise<void>;
  modId?: string | null;
  previewSyncRevisionRef: MutableRefObject<number>;
  recordEvent: (event: EditorEvent) => void;
  regeneratePreview: (modId: string, sceneId: string, force?: boolean) => Promise<void>;
  refreshEditorSnapshotForScene: (scene: EditorSceneSummaryDto | null) => Promise<void>;
  selectedScene: EditorSceneSummaryDto | null;
  sessionId?: string | null;
  setEditorPreviewSync: (sync: SceneEditorPreviewSyncState) => void;
  setEditorSnapshot: (snapshot: EditorSceneSnapshotDto | null) => void;
  setEditorSnapshotSceneId: (sceneId: string | null) => void;
}) {
  const regeneratePreviewForEditorCommand = useCallback(
    async ({ revision, sceneId }: { revision: number; sceneId: string }) => {
      if (!modId) return;

      try {
        await regeneratePreview(modId, sceneId, true);
        if (previewSyncRevisionRef.current !== revision) return;
        setEditorPreviewSync(sceneEditorPreviewReady({ sceneId, revision }));
        recordEvent({
          type: "EditorPreviewRegenerated",
          sceneId,
          revision,
        });
      } catch (reason) {
        if (previewSyncRevisionRef.current !== revision) return;
        const message = reason instanceof Error ? reason.message : String(reason);
        setEditorPreviewSync(sceneEditorPreviewFailed({ sceneId, revision, message }));
        recordEvent({
          type: "EditorPreviewRegenerationFailed",
          sceneId,
          revision,
          error: message,
        });
      }
    },
    [modId, previewSyncRevisionRef, recordEvent, regeneratePreview, setEditorPreviewSync],
  );

  return useCallback(
    async (command: EditorCommandDto): Promise<EditorCommandResultDto | null> => {
      if (!sessionId) return null;

      try {
        if (editorModeSessionId && commandChangesUiStructure(command)) {
          const frameResult = await applyEditorModeCommandApi(sessionId, editorModeSessionId, command);
          applyEditorFrameResult(frameResult);
          if (frameResult.ok && commandChangesUiStructure(command)) {
            await loadEditorModeSceneHierarchy(sessionId, editorModeSessionId);
          }
          return {
            ok: frameResult.ok,
            sceneDirty: frameResult.session?.dirty ?? true,
            changedEntities: changedEntitiesForCommand(command),
            snapshot: frameResult.snapshot ?? undefined,
            diagnostics: frameResult.diagnostics,
            message: frameResult.message ?? undefined,
          };
        }

        const result = await applyEditorCommandApi(sessionId, command);
        if (result.snapshot) {
          setEditorSnapshot(result.snapshot);
          setEditorSnapshotSceneId(result.snapshot.sceneId);
        }

        if (
          result.ok &&
          modId &&
          commandChangesSceneDocument(command)
        ) {
          if (commandChangesUiStructure(command)) {
            await loadSceneHierarchy(modId, command.sceneId, true);
          }
          if (!result.snapshot && selectedScene?.id === command.sceneId) {
            await refreshEditorSnapshotForScene(selectedScene);
          }
          const revision = previewSyncRevisionRef.current + 1;
          previewSyncRevisionRef.current = revision;
          setEditorPreviewSync(sceneEditorPreviewRegenerating({
            sceneId: command.sceneId,
            revision,
          }));
          void regeneratePreviewForEditorCommand({
            sceneId: command.sceneId,
            revision,
          });
        }

        return result;
      } catch (reason) {
        recordEvent({
          type: "EditorCommandFailed",
          command: command.type,
          error: reason instanceof Error ? reason.message : String(reason),
        });
        return null;
      }
    },
    [
      applyEditorFrameResult,
      editorModeSessionId,
      loadEditorModeSceneHierarchy,
      loadSceneHierarchy,
      modId,
      previewSyncRevisionRef,
      recordEvent,
      refreshEditorSnapshotForScene,
      regeneratePreviewForEditorCommand,
      selectedScene,
      sessionId,
      setEditorPreviewSync,
      setEditorSnapshot,
      setEditorSnapshotSceneId,
    ],
  );
}
