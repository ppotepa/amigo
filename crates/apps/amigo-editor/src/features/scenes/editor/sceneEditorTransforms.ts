import type {
  SceneEditorPoint,
  SceneEditorResolution,
  SceneEditorViewportState,
} from "./sceneEditorTypes";

export const MIN_SCENE_EDITOR_ZOOM = 0.1;
export const MAX_SCENE_EDITOR_ZOOM = 6;

export function clampZoom(value: number): number {
  return Math.max(MIN_SCENE_EDITOR_ZOOM, Math.min(MAX_SCENE_EDITOR_ZOOM, value));
}

export function zoomPercent(zoom: number): string {
  return `${Math.round(zoom * 100)}%`;
}

export function sceneToScreen(
  point: SceneEditorPoint,
  viewport: SceneEditorViewportState,
): SceneEditorPoint {
  return {
    x: point.x * viewport.zoom + viewport.panX,
    y: point.y * viewport.zoom + viewport.panY,
  };
}

export function screenToScene(
  point: SceneEditorPoint,
  viewport: SceneEditorViewportState,
): SceneEditorPoint {
  return {
    x: (point.x - viewport.panX) / viewport.zoom,
    y: (point.y - viewport.panY) / viewport.zoom,
  };
}

export function fitResolutionToViewport(
  resolution: SceneEditorResolution,
  container: { width: number; height: number },
  padding = 80,
): SceneEditorViewportState {
  const availableWidth = Math.max(1, container.width - padding);
  const availableHeight = Math.max(1, container.height - padding);
  const zoom = clampZoom(Math.min(
    availableWidth / resolution.width,
    availableHeight / resolution.height,
  ));

  return {
    zoom,
    panX: Math.round((container.width - resolution.width * zoom) / 2),
    panY: Math.round((container.height - resolution.height * zoom) / 2),
  };
}

export function zoomAroundScreenPoint({
  nextZoom,
  point,
  viewport,
}: {
  viewport: SceneEditorViewportState;
  point: SceneEditorPoint;
  nextZoom: number;
}): SceneEditorViewportState {
  const clamped = clampZoom(nextZoom);
  const scenePoint = screenToScene(point, viewport);

  return {
    zoom: clamped,
    panX: point.x - scenePoint.x * clamped,
    panY: point.y - scenePoint.y * clamped,
  };
}

export function translateViewport(
  viewport: SceneEditorViewportState,
  delta: SceneEditorPoint,
): SceneEditorViewportState {
  return {
    ...viewport,
    panX: viewport.panX + delta.x,
    panY: viewport.panY + delta.y,
  };
}
