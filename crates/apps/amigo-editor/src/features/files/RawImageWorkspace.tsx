import { useRef, useState } from "react";
import type { EditorProjectFileDto } from "../../api/dto";
import { fileSrc } from "../../utils/fileSrc";

export function RawImageWorkspace({ file }: { file: EditorProjectFileDto }) {
  const [zoom, setZoom] = useState(1);
  const [fitMode, setFitMode] = useState(true);
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<{ active: boolean; left: number; top: number; x: number; y: number }>({
    active: false,
    left: 0,
    top: 0,
    x: 0,
    y: 0,
  });

  function adjustZoom(delta: number) {
    setFitMode(false);
    setZoom((current) => Math.min(5, Math.max(0.1, Number((current + delta).toFixed(2)))));
  }

  function resetPan() {
    if (!viewportRef.current) return;
    viewportRef.current.scrollLeft = 0;
    viewportRef.current.scrollTop = 0;
  }

  return (
    <div
      className="file-image-stage"
      onWheel={(event) => {
        if (!event.ctrlKey) return;
        event.preventDefault();
        adjustZoom(event.deltaY > 0 ? -0.1 : 0.1);
      }}
    >
      <div className="file-image-toolbar">
        <button className="button button-icon" type="button" title="Zoom out" onClick={() => adjustZoom(-0.25)}>
          -
        </button>
        <span className="file-image-zoom-label">{Math.round(zoom * 100)}%</span>
        <button className="button button-icon" type="button" title="Fit image" onClick={() => { setFitMode(true); setZoom(1); resetPan(); }}>
          Fit
        </button>
        <button className="button button-icon" type="button" title="Reset zoom" onClick={() => { setFitMode(false); setZoom(1); resetPan(); }}>
          1:1
        </button>
        <button className="button button-icon" type="button" title="Reset pan" onClick={resetPan}>
          Pan
        </button>
        <button className="button button-icon" type="button" title="Zoom in" onClick={() => adjustZoom(0.25)}>
          +
        </button>
      </div>
      <div
        ref={viewportRef}
        className="file-image-viewport"
        onMouseDown={(event) => {
          if (event.button !== 0 && event.button !== 1) return;
          dragRef.current = {
            active: true,
            left: viewportRef.current?.scrollLeft ?? 0,
            top: viewportRef.current?.scrollTop ?? 0,
            x: event.clientX,
            y: event.clientY,
          };
        }}
        onMouseMove={(event) => {
          if (!dragRef.current.active || !viewportRef.current) return;
          viewportRef.current.scrollLeft = dragRef.current.left - (event.clientX - dragRef.current.x);
          viewportRef.current.scrollTop = dragRef.current.top - (event.clientY - dragRef.current.y);
        }}
        onMouseUp={() => {
          dragRef.current.active = false;
        }}
        onMouseLeave={() => {
          dragRef.current.active = false;
        }}
      >
        <img
          className="file-image-preview"
          src={fileSrc(file.path)}
          alt={file.name}
          draggable={false}
          style={{
            maxWidth: fitMode ? "100%" : "none",
            maxHeight: fitMode ? "100%" : "none",
            transform: fitMode ? undefined : `scale(${zoom})`,
          }}
        />
      </div>
    </div>
  );
}
