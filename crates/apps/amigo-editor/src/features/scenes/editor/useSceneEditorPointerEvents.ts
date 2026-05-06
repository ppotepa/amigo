import { useState } from "react";
import type React from "react";
import type { EditorFrameResultDto, EditorPointerEventDto } from "../../../api/dto";
import type {
  SceneEditorCanvasProps,
  SceneEditorModel,
  SceneEditorPoint,
  SceneEditorViewportState,
} from "./sceneEditorTypes";
import { clampZoom, screenToArtboard, zoomAroundScreenPoint } from "./sceneEditorTransforms";
import { buildEditorViewport } from "./useSceneEditorViewportResize";

function localPoint(event: { currentTarget: Element; clientX: number; clientY: number }): SceneEditorPoint {
  const rect = event.currentTarget.getBoundingClientRect();
  return {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  };
}

function isEditorChromeEvent(event: React.SyntheticEvent<HTMLElement>): boolean {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return false;
  return Boolean(
    target.closest([
      "button",
      "input",
      "select",
      "textarea",
      "[data-editor-chrome='true']",
    ].join(",")),
  );
}

function frameToScene(
  framePoint: SceneEditorPoint,
  model: SceneEditorModel,
): SceneEditorPoint {
  const zoom = Math.max(0.0001, model.camera.zoom || 1);
  return {
    x: ((framePoint.x - model.resolution.width * 0.5) / zoom) + model.camera.x,
    y: ((model.resolution.height * 0.5 - framePoint.y) / zoom) + model.camera.y,
  };
}

export function useSceneEditorPointerEvents({
  model,
  onPointerEvent,
  onViewportChange,
  viewport,
}: {
  model: SceneEditorModel;
  onPointerEvent?: (event: EditorPointerEventDto) => Promise<EditorFrameResultDto | null>;
  onViewportChange: (viewport: SceneEditorViewportState) => void;
  viewport: SceneEditorViewportState;
}) {
  const [engineHover, setEngineHover] = useState(false);
  const [mouseScenePoint, setMouseScenePoint] = useState<SceneEditorPoint | null>(null);

  function updateMousePoint(event: React.PointerEvent<HTMLDivElement>) {
    setMouseScenePoint(frameToScene(screenToArtboard(localPoint(event), viewport), model));
  }

  function toEditorPointerEvent(
    event: React.PointerEvent<HTMLDivElement>,
    type: EditorPointerEventDto["type"],
  ): EditorPointerEventDto {
    const screenPoint = localPoint(event);
    const framePoint = screenToArtboard(screenPoint, viewport);
    const scenePoint = frameToScene(framePoint, model);
    const editorViewport = buildEditorViewport(event.currentTarget);

    return {
      type,
      x: scenePoint.x,
      y: scenePoint.y,
      sceneX: scenePoint.x,
      sceneY: scenePoint.y,
      frameX: framePoint.x,
      frameY: framePoint.y,
      button: event.button,
      buttons: event.buttons,
      pointerId: event.pointerId,
      modifiers: {
        shift: event.shiftKey,
        ctrl: event.ctrlKey,
        alt: event.altKey,
        meta: event.metaKey,
      },
      viewport: editorViewport,
    };
  }

  async function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerDown"));
  }

  async function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerMove"));
  }

  async function handlePointerUp(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerUp"));
  }

  async function handlePointerCancel(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerCancel"));
  }

  function handlePointerEnter(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    setEngineHover(true);
  }

  async function handlePointerLeave(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    setEngineHover(false);
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerCancel"));
  }

  async function handleWheel(event: React.WheelEvent<HTMLDivElement>) {
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      const point = localPoint(event);
      const factor = event.deltaY > 0 ? 0.9 : 1.1;
      onViewportChange(zoomAroundScreenPoint({
        viewport,
        point,
        nextZoom: clampZoom(viewport.zoom * factor),
      }));
    }
  }

  return {
    engineHover,
    handlePointerCancel,
    handlePointerDown,
    handlePointerEnter,
    handlePointerLeave,
    handlePointerMove,
    handlePointerUp,
    handleWheel,
    mouseScenePoint,
  };
}
