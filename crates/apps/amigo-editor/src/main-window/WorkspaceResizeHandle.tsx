import { useRef } from "react";
import type React from "react";

export function WorkspaceResizeHandle({
  className,
  onDrag,
  onReset,
  orientation,
  title,
}: {
  className: string;
  onDrag: (delta: number) => void;
  onReset: () => void;
  orientation: "vertical" | "horizontal";
  title: string;
}) {
  const dragRef = useRef<{ pointerId: number; x: number; y: number } | null>(null);

  function handlePointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
    document.body.classList.add("workspace-resizing");
  }

  function handlePointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const delta = orientation === "vertical" ? event.clientX - drag.x : event.clientY - drag.y;
    if (delta === 0) return;
    onDrag(delta);
    dragRef.current = { pointerId: event.pointerId, x: event.clientX, y: event.clientY };
  }

  function endDrag(event: React.PointerEvent<HTMLButtonElement>) {
    if (dragRef.current?.pointerId === event.pointerId) {
      dragRef.current = null;
      document.body.classList.remove("workspace-resizing");
    }
  }

  return (
    <button
      aria-label={title}
      className={`workspace-resize-handle ${orientation} ${className}`}
      title={`${title}. Double click to reset.`}
      type="button"
      onDoubleClick={onReset}
      onPointerCancel={endDrag}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={endDrag}
    />
  );
}
