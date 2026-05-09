import { useRef, useState } from "react";
import type { ReactNode } from "react";

type SplitPaneProps = {
  top?: ReactNode;
  bottom?: ReactNode;
  first?: ReactNode;
  second?: ReactNode;
  defaultRatio?: number;
  defaultTopRatio?: number;
  topRatio?: number;
  minRatio?: number;
  maxRatio?: number;
  firstMin?: number;
  firstMax?: number;
};

export function SplitPane({
  top,
  bottom,
  first,
  second,
  defaultTopRatio,
  defaultRatio,
  topRatio: topRatioFromProps,
  minRatio,
  maxRatio,
  firstMin,
  firstMax,
}: SplitPaneProps) {
  const min = firstMin ?? minRatio ?? 0.28;
  const max = firstMax ?? maxRatio ?? 0.76;
  const defaultValue = topRatioFromProps ?? defaultRatio ?? defaultTopRatio ?? 0.54;

  const [topPanelRatio, setTopPanelRatio] = useState(defaultValue);
  const containerRef = useRef<HTMLDivElement | null>(null);

  const effectiveTop = top ?? first;
  const effectiveBottom = bottom ?? second;

  function handlePointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;

    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect || rect.height <= 0) return;

    const nextRatio = (event.clientY - rect.top) / rect.height;
    setTopPanelRatio(Math.min(max, Math.max(min, nextRatio)));
  }

  return (
    <div className="workbench-split-pane" ref={containerRef}>
      <div
        className="workbench-split-pane-top"
        style={{
          flexBasis: `${topPanelRatio * 100}%`,
          minHeight: 0,
          flexShrink: 0,
          overflow: "hidden",
        }}
      >
        {effectiveTop}
      </div>
      <button
        type="button"
        className="workbench-splitter"
        aria-label="Resize split pane"
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
      />
      <div className="workbench-split-pane-bottom" style={{ flex: "1 1 auto", minHeight: 0 }}>
        {effectiveBottom}
      </div>
    </div>
  );
}
