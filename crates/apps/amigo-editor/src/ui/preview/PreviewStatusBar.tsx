import type { ReactNode } from "react";

export function PreviewStatusBar({
  left,
  right,
}: {
  left: ReactNode;
  right?: ReactNode;
}) {
  return (
    <footer className="preview-statusbar">
      <span>{left}</span>
      {right ? <span>{right}</span> : null}
    </footer>
  );
}
