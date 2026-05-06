import type { ReactNode } from "react";
import { ChevronDown, ChevronRight, MoreVertical } from "lucide-react";
import type { ContextBadgeTone } from "./contextDockTypes";

export function ContextWidgetHeader({
  actions,
  badge,
  badgeTone = "muted",
  collapsed,
  icon,
  onToggle,
  title,
}: {
  actions?: ReactNode;
  badge?: ReactNode;
  badgeTone?: ContextBadgeTone;
  collapsed: boolean;
  icon?: ReactNode;
  onToggle: () => void;
  title: string;
}) {
  const ToggleIcon = collapsed ? ChevronRight : ChevronDown;

  return (
    <header className="context-widget-header">
      <button
        className="context-widget-toggle"
        type="button"
        title={collapsed ? "Expand" : "Collapse"}
        aria-label={collapsed ? "Expand widget" : "Collapse widget"}
        onClick={onToggle}
      >
        <ToggleIcon size={14} />
      </button>

      {icon ? <span className="context-widget-icon">{icon}</span> : null}

      <strong className="context-widget-title">
        {title}
      </strong>

      {badge != null ? (
        <span className={`context-widget-badge context-widget-badge-${badgeTone}`}>
          {badge}
        </span>
      ) : null}

      <div className="context-widget-header-spacer" />

      {actions}

      <button className="context-widget-menu" type="button" title="More actions" aria-label="More actions">
        <MoreVertical size={14} />
      </button>
    </header>
  );
}
