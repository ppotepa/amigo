import { useCallback } from "react";
import type { MutableRefObject } from "react";
import {
  discardEditorModeSessionChanges as discardEditorModeSessionChangesApi,
  redoEditorModeTransaction as redoEditorModeTransactionApi,
  resizeEditorModeViewport as resizeEditorModeViewportApi,
  saveEditorModeSession as saveEditorModeSessionApi,
  sendEditorPointerEvent as sendEditorPointerEventApi,
  setEditorMode as setEditorModeApi,
  setEditorTool as setEditorToolApi,
  undoEditorModeTransaction as undoEditorModeTransactionApi,
} from "../../api/editorApi";
import type {
  EditorFrameResultDto,
  EditorModeDto,
  EditorModeSessionDto,
  EditorPointerEventDto,
  EditorToolDto,
  EditorViewportDto,
} from "../../api/dto";
import type { EditorEvent } from "../../app/editorEvents";

type EditorCommandName =
  | "DiscardEditorModeSessionChanges"
  | "RedoEditorModeTransaction"
  | "ResizeEditorModeViewport"
  | "SaveEditorModeSession"
  | "SendEditorPointerEvent"
  | "SetEditorMode"
  | "SetEditorTool"
  | "UndoEditorModeTransaction";

export function useEditorModeCommands({
  applyEditorFrameResult,
  editorModeSessionRef,
  loadEditorModeSceneHierarchy,
  recordEvent,
  sessionId,
}: {
  applyEditorFrameResult: (result: EditorFrameResultDto | null | undefined) => void;
  editorModeSessionRef: MutableRefObject<EditorModeSessionDto | null>;
  loadEditorModeSceneHierarchy?: (sessionId: string, editorModeSessionId: string) => Promise<void>;
  recordEvent: (event: EditorEvent) => void;
  sessionId?: string | null;
}) {
  const recordFailure = useCallback(
    (command: EditorCommandName, reason: unknown) => {
      recordEvent({
        type: "EditorCommandFailed",
        command,
        error: reason instanceof Error ? reason.message : String(reason),
      });
    },
    [recordEvent],
  );

  const saveEditorModeSession = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!sessionId || !currentSession) return;
    try {
      const result = await saveEditorModeSessionApi(sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
      await loadEditorModeSceneHierarchy?.(sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordFailure("SaveEditorModeSession", reason);
    }
  }, [applyEditorFrameResult, editorModeSessionRef, loadEditorModeSceneHierarchy, recordFailure, sessionId]);

  const discardEditorModeSessionChanges = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!sessionId || !currentSession) return;
    try {
      const result = await discardEditorModeSessionChangesApi(sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
      await loadEditorModeSceneHierarchy?.(sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordFailure("DiscardEditorModeSessionChanges", reason);
    }
  }, [applyEditorFrameResult, editorModeSessionRef, loadEditorModeSceneHierarchy, recordFailure, sessionId]);

  const undoEditorModeTransaction = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!sessionId || !currentSession) return;
    try {
      const result = await undoEditorModeTransactionApi(sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
      await loadEditorModeSceneHierarchy?.(sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordFailure("UndoEditorModeTransaction", reason);
    }
  }, [applyEditorFrameResult, editorModeSessionRef, loadEditorModeSceneHierarchy, recordFailure, sessionId]);

  const redoEditorModeTransaction = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!sessionId || !currentSession) return;
    try {
      const result = await redoEditorModeTransactionApi(sessionId, currentSession.editorModeSessionId);
      applyEditorFrameResult(result);
      await loadEditorModeSceneHierarchy?.(sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordFailure("RedoEditorModeTransaction", reason);
    }
  }, [applyEditorFrameResult, editorModeSessionRef, loadEditorModeSceneHierarchy, recordFailure, sessionId]);

  const resizeEditorModeViewport = useCallback(
    async (viewport: EditorViewportDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!sessionId || !currentSession) return null;
      try {
        const result = await resizeEditorModeViewportApi(
          sessionId,
          currentSession.editorModeSessionId,
          viewport,
        );
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordFailure("ResizeEditorModeViewport", reason);
        return null;
      }
    },
    [applyEditorFrameResult, editorModeSessionRef, recordFailure, sessionId],
  );

  const setEditorMode = useCallback(
    async (mode: EditorModeDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!sessionId || !currentSession) return null;
      try {
        const result = await setEditorModeApi(sessionId, currentSession.editorModeSessionId, mode);
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordFailure("SetEditorMode", reason);
        return null;
      }
    },
    [applyEditorFrameResult, editorModeSessionRef, recordFailure, sessionId],
  );

  const setEditorTool = useCallback(
    async (tool: EditorToolDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!sessionId || !currentSession) return null;
      try {
        const result = await setEditorToolApi(sessionId, currentSession.editorModeSessionId, tool);
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordFailure("SetEditorTool", reason);
        return null;
      }
    },
    [applyEditorFrameResult, editorModeSessionRef, recordFailure, sessionId],
  );

  const sendEditorPointerEvent = useCallback(
    async (event: EditorPointerEventDto): Promise<EditorFrameResultDto | null> => {
      const currentSession = editorModeSessionRef.current;
      if (!sessionId || !currentSession) return null;
      try {
        const result = await sendEditorPointerEventApi(sessionId, currentSession.editorModeSessionId, event);
        applyEditorFrameResult(result);
        return result;
      } catch (reason) {
        recordFailure("SendEditorPointerEvent", reason);
        return null;
      }
    },
    [applyEditorFrameResult, editorModeSessionRef, recordFailure, sessionId],
  );

  return {
    discardEditorModeSessionChanges,
    redoEditorModeTransaction,
    resizeEditorModeViewport,
    saveEditorModeSession,
    sendEditorPointerEvent,
    setEditorMode,
    setEditorTool,
    undoEditorModeTransaction,
  };
}
