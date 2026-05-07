import type { CSSProperties, ReactNode } from "react";

function cx(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

export function PreviewArtboard({
  children,
  className,
  chrome = true,
  height,
  panX = 0,
  panY = 0,
  style,
  width,
  zoom,
}: {
  children: ReactNode;
  className?: string;
  chrome?: boolean;
  height: number;
  panX?: number;
  panY?: number;
  style?: CSSProperties;
  width: number;
  zoom: number;
}) {
  return (
    <div
      className={cx(chrome ? "preview-artboard" : "", className)}
      style={{
        ...style,
        width,
        height,
        transform: `translate(${panX}px, ${panY}px) scale(${zoom})`,
      }}
    >
      {children}
    </div>
  );
}
