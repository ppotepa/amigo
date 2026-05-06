import { useCallback, useEffect, useRef, useState } from "react";
import {
  closeEditorModeSession as closeEditorModeSessionApi,
  getEditorSceneSnapshot,
  openEditorModeSession as openEditorModeSessionApi,
} from "../../api/editorApi";
import type {
  EditorFrameDto,
  EditorModeSessionDto,
  EditorSceneSnapshotDto,
  EditorSceneSummaryDto,
} from "../../api/dto";
import type { EditorEvent } from "../../app/editorEvents";

export function useEditorModeSessionLifecycle({
  recordEvent,
  scene,
  sessionId,
  setEditorFrame,
  setEditorModeSession,
  setEditorSnapshot,
  setEditorSnapshotSceneId,
}: {
  recordEvent: (event: EditorEvent) => void;
  scene: EditorSceneSummaryDto | null;
  sessionId?: string | null;
  setEditorFrame: (frame: EditorFrameDto | null) => void;
  setEditorModeSession: (session: EditorModeSessionDto | null) => void;
  setEditorSnapshot: (snapshot: EditorSceneSnapshotDto | null) => void;
  setEditorSnapshotSceneId: (sceneId: string | null) => void;
}) {
  const [editorModeOpening, setEditorModeOpening] = useState(false);
  const [editorModeError, setEditorModeError] = useState<string | null>(null);
  const editorModeSessionRef = useRef<EditorModeSessionDto | null>(null);
  const openingEditorModeSceneRef = useRef<string | null>(null);

  const resetEditorModeSession = useCallback(() => {
    editorModeSessionRef.current = null;
    openingEditorModeSceneRef.current = null;
    setEditorModeSession(null);
    setEditorFrame(null);
    setEditorModeOpening(false);
    setEditorModeError(null);
  }, [setEditorFrame, setEditorModeSession]);

  const refreshEditorSnapshotForScene = useCallback(
    async (targetScene: EditorSceneSummaryDto | null) => {
      if (!sessionId || !targetScene) {
        setEditorSnapshot(null);
        setEditorSnapshotSceneId(null);
        return;
      }

      try {
        const snapshot = await getEditorSceneSnapshot(sessionId, targetScene.id);
        setEditorSnapshot(snapshot);
        setEditorSnapshotSceneId(targetScene.id);
        recordEvent({
          type: "EditorSnapshotLoaded",
          sceneId: targetScene.id,
          objects: snapshot.objects.length,
        });
      } catch (reason) {
        setEditorSnapshot(null);
        setEditorSnapshotSceneId(null);
        recordEvent({
          type: "EditorSnapshotUnavailable",
          sceneId: targetScene.id,
          error: reason instanceof Error ? reason.message : String(reason),
        });
      }
    },
    [recordEvent, sessionId, setEditorSnapshot, setEditorSnapshotSceneId],
  );

  const refreshEditorSnapshot = useCallback(async () => {
    await refreshEditorSnapshotForScene(scene);
  }, [refreshEditorSnapshotForScene, scene]);

  const closeEditorModeSession = useCallback(async () => {
    const currentSession = editorModeSessionRef.current;
    if (!sessionId || !currentSession) {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      return;
    }

    try {
      await closeEditorModeSessionApi(sessionId, currentSession.editorModeSessionId);
    } catch (reason) {
      recordEvent({
        type: "EditorCommandFailed",
        command: "CloseEditorModeSession",
        error: reason instanceof Error ? reason.message : String(reason),
      });
    } finally {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      setEditorModeOpening(false);
      setEditorModeError(null);
    }
  }, [recordEvent, sessionId, setEditorFrame, setEditorModeSession]);

  const openEditorModeSession = useCallback(async () => {
    const sceneId = scene?.id;
    if (!sessionId || !sceneId) return;
    if (editorModeSessionRef.current?.sceneId === sceneId) return;
    if (openingEditorModeSceneRef.current === sceneId) return;

    openingEditorModeSceneRef.current = sceneId;
    setEditorModeOpening(true);
    setEditorModeError(null);

    try {
      const devicePixelRatio = window.devicePixelRatio || 1;
      const result = await openEditorModeSessionApi(sessionId, sceneId, {
        cssWidth: 1280,
        cssHeight: 720,
        renderWidth: Math.round(1280 * devicePixelRatio),
        renderHeight: Math.round(720 * devicePixelRatio),
        devicePixelRatio,
      });
      editorModeSessionRef.current = result.session;
      setEditorModeSession(result.session);
      setEditorFrame(result.frame);
      setEditorSnapshot(result.snapshot);
      setEditorSnapshotSceneId(result.snapshot.sceneId);
    } catch (reason) {
      editorModeSessionRef.current = null;
      setEditorModeSession(null);
      setEditorFrame(null);
      setEditorModeError(reason instanceof Error ? reason.message : String(reason));
      recordEvent({
        type: "EditorSnapshotUnavailable",
        sceneId,
        error: reason instanceof Error ? reason.message : String(reason),
      });
    } finally {
      if (openingEditorModeSceneRef.current === sceneId) {
        openingEditorModeSceneRef.current = null;
      }
      setEditorModeOpening(false);
    }
  }, [
    recordEvent,
    scene?.id,
    sessionId,
    setEditorFrame,
    setEditorModeSession,
    setEditorSnapshot,
    setEditorSnapshotSceneId,
  ]);

  useEffect(() => {
    void refreshEditorSnapshotForScene(scene);
  }, [refreshEditorSnapshotForScene, scene?.id]);

  useEffect(() => {
    if (!sessionId || !scene?.id) {
      resetEditorModeSession();
      return;
    }
    void openEditorModeSession();
  }, [openEditorModeSession, resetEditorModeSession, scene?.id, sessionId]);

  return {
    closeEditorModeSession,
    editorModeError,
    editorModeOpening,
    editorModeSessionRef,
    openEditorModeSession,
    refreshEditorSnapshot,
    refreshEditorSnapshotForScene,
  };
}
