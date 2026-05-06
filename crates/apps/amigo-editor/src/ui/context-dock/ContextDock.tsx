import type { ReactNode } from "react";

export function ContextDock({ children, empty }: { children: ReactNode; empty?: ReactNode }) {
  if (!children) {
    return (
      <div className="context-dock context-dock-empty">
        {empty ?? <p className="muted workspace-empty">No context selected.</p>}
      </div>
    );
  }

  return <div className="context-dock">{children}</div>;
}
