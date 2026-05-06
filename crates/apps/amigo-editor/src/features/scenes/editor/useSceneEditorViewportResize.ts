import { useCallback, useEffect, useRef } from "react";
import type { EditorFrameResultDto, EditorModeSessionDto, EditorViewportDto } from "../../../api/dto";
import type { SceneEditorResolution, SceneEditorViewportState } from "./sceneEditorTypes";
import { fitResolutionToViewport } from "./sceneEditorTransforms";

function sameEditorViewport(a: EditorViewportDto, b: EditorViewportDto): boolean {
  return (
    Math.abs(a.cssWidth - b.cssWidth) < 0.5 &&
    Math.abs(a.cssHeight - b.cssHeight) < 0.5 &&
    a.renderWidth === b.renderWidth &&
    a.renderHeight === b.renderHeight &&
    Math.abs(a.devicePixelRatio - b.devicePixelRatio) < 0.001 &&
    Math.abs((a.cameraX ?? 0) - (b.cameraX ?? 0)) < 0.001 &&
    Math.abs((a.cameraY ?? 0) - (b.cameraY ?? 0)) < 0.001 &&
    Math.abs((a.zoom ?? 1) - (b.zoom ?? 1)) < 0.001
  );
}

export function buildEditorViewport(element: HTMLElement): EditorViewportDto {
  const rect = element.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  return {
    cssWidth: rect.width,
    cssHeight: rect.height,
    renderWidth: Math.max(1, Math.round(rect.width * dpr)),
    renderHeight: Math.max(1, Math.round(rect.height * dpr)),
    devicePixelRatio: dpr,
    cameraX: 0,
    cameraY: 0,
    zoom: 1,
  };
}

export function useSceneEditorViewportResize({
  containerRef,
  editorModeSession,
  onViewportChange,
  onViewportResize,
  resolution,
  sceneId,
}: {
  containerRef: React.RefObject<HTMLDivElement | null>;
  editorModeSession?: EditorModeSessionDto | null;
  onViewportChange: (viewport: SceneEditorViewportState) => void;
  onViewportResize?: (viewport: EditorViewportDto) => Promise<EditorFrameResultDto | null>;
  resolution: SceneEditorResolution;
  sceneId: string;
}) {
  const lastEditorViewportRef = useRef<EditorViewportDto | null>(null);

  useEffect(() => {
    const element = containerRef.current;
    if (!element) return;
    const rect = element.getBoundingClientRect();
    onViewportChange(fitResolutionToViewport(resolution, {
      width: rect.width,
      height: rect.height,
    }));
  }, [containerRef, onViewportChange, resolution.height, resolution.width, sceneId]);

  const emitViewportResize = useCallback((element: HTMLElement) => {
    if (!onViewportResize) return;
    const nextViewport = buildEditorViewport(element);
    const previousViewport = lastEditorViewportRef.current;
    if (previousViewport && sameEditorViewport(previousViewport, nextViewport)) {
      return;
    }
    lastEditorViewportRef.current = nextViewport;
    void onViewportResize(nextViewport);
  }, [onViewportResize]);

  useEffect(() => {
    const element = containerRef.current;
    if (!element || !onViewportResize || !editorModeSession) return;

    let raf = 0;
    lastEditorViewportRef.current = null;
    const resizeObserver = new ResizeObserver(() => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        emitViewportResize(element);
      });
    });

    resizeObserver.observe(element);
    emitViewportResize(element);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver.disconnect();
    };
  }, [
    containerRef,
    editorModeSession?.editorModeSessionId,
    emitViewportResize,
    onViewportResize,
  ]);
}
