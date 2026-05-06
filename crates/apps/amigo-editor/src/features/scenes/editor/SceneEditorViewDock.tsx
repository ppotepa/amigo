import { Maximize2, Minus, Plus } from "lucide-react";
import type React from "react";
import { clampZoom } from "./sceneEditorTransforms";

export function SceneEditorViewDock({
  zoom,
  onFitViewport,
  onResetZoom,
  onZoomChange,
  onZoomIn,
  onZoomOut,
}: {
  zoom: number;
  onFitViewport: () => void;
  onResetZoom: () => void;
  onZoomChange: (zoom: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
}) {
  const zoomMin = 0.25;
  const zoomMax = 4;
  const normalizedZoom = Math.max(0, Math.min(1, (zoom - zoomMin) / (zoomMax - zoomMin)));

  function zoomFromPointer(event: React.PointerEvent<HTMLButtonElement>) {
    const rect = event.currentTarget.getBoundingClientRect();
    const ratio = 1 - ((event.clientY - rect.top) / rect.height);
    const nextZoom = zoomMin + Math.max(0, Math.min(1, ratio)) * (zoomMax - zoomMin);
    onZoomChange(clampZoom(nextZoom));
  }

  return (
    <div
      className="scene-editor-floating-dock scene-editor-view-dock"
      aria-label="Viewport controls"
      data-editor-chrome="true"
      onPointerDown={(event) => event.stopPropagation()}
      onPointerMove={(event) => event.stopPropagation()}
      onPointerUp={(event) => event.stopPropagation()}
      onClick={(event) => event.stopPropagation()}
    >
      <button className="scene-editor-floating-button" type="button" title="Zoom in" onClick={onZoomIn}>
        <Plus size={15} />
      </button>
      <div className="scene-editor-zoom-slider" title={`Zoom ${Math.round(zoom * 100)}%`}>
        <span>{Math.round(zoom * 100)}%</span>
        <button
          aria-label="Zoom"
          className="scene-editor-zoom-slider-track"
          type="button"
          onDoubleClick={onResetZoom}
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            zoomFromPointer(event);
          }}
          onPointerMove={(event) => {
            if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
            zoomFromPointer(event);
          }}
        >
          <span className="scene-editor-zoom-slider-fill" style={{ height: `${normalizedZoom * 100}%` }} />
          <span className="scene-editor-zoom-slider-thumb" style={{ bottom: `${normalizedZoom * 100}%` }} />
        </button>
      </div>
      <button className="scene-editor-floating-button" type="button" title="Zoom out" onClick={onZoomOut}>
        <Minus size={15} />
      </button>
      <span className="scene-editor-floating-separator" />
      <button className="scene-editor-floating-button" type="button" title="Fit to view" onClick={onFitViewport}>
        <Maximize2 size={15} />
      </button>
    </div>
  );
}
