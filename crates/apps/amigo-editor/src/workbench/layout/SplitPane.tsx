import type { ReactNode } from "react";
import { useRef, useState } from "react";

export function SplitPane({
  bottom,
  defaultTopRatio = 0.54,
  top,
}: {
  top?: ReactNode;
  bottom?: ReactNode;
  defaultTopRatio?: number;
}) {
  const [topRatio, setTopRatio] = useState(defaultTopRatio);
  const containerRef = useRef<HTMLDivElement | null>(null);

  function handlePointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect || rect.height <= 0) return;
    const nextRatio = (event.clientY - rect.top) / rect.height;
    setTopRatio(Math.min(0.76, Math.max(0.28, nextRatio)));
  }

  return (
    <div className="workbench-split-pane" ref={containerRef}>
      <div className="workbench-split-pane-top" style={{ flexBasis: `${topRatio * 100}%` }}>
        {top}
      </div>
      <button
        type="button"
        className="workbench-splitter"
        aria-label="Resize split pane"
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
      />
      <div className="workbench-split-pane-bottom">{bottom}</div>
    </div>
  );
}
