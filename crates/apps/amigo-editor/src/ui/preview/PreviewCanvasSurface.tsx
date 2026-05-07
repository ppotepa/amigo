import type { ReactNode } from "react";

function cx(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

export function PreviewCanvasSurface({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cx("preview-canvas-surface", className)}>{children}</div>;
}
