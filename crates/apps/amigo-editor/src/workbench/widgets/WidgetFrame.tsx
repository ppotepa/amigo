import type { CSSProperties, ReactNode } from "react";
import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import type { ContextBadgeTone } from "../../ui/context-dock/contextDockTypes";
import type { WidgetStatus } from "./widgetTypes";

export type WidgetFrameProps = {
  id: string;
  title: string;
  status?: WidgetStatus;
  icon?: ReactNode;
  badge?: ReactNode;
  badgeTone?: ContextBadgeTone;
  compact?: boolean;
  foldedHint?: string;
  defaultCollapsed?: boolean;
  actions?: ReactNode;
  bodyClassName?: string;
  className?: string;
  maxBodyHeight?: number | string;
  scrollable?: boolean;
  children: ReactNode;
};

export function WidgetFrame({
  actions,
  badge,
  badgeTone = "muted",
  bodyClassName = "",
  children,
  className = "",
  compact = false,
  defaultCollapsed,
  foldedHint,
  id,
  icon,
  maxBodyHeight,
  scrollable = false,
  status,
  title,
}: WidgetFrameProps) {
  const [collapsed, setCollapsed] = useState(Boolean(defaultCollapsed));
  const ToggleIcon = collapsed ? ChevronRight : ChevronDown;

  return (
    <section
      className={`context-widget workbench-widget-frame ${compact ? "compact" : ""} ${collapsed ? "folded collapsed" : ""} ${className}`}
      data-widget-id={id}
    >
      <header className={`context-widget-header ${compact ? "context-widget-header-compact" : ""}`}>
        <button
          className="context-widget-toggle"
          type="button"
          title={collapsed ? "Expand" : "Collapse"}
          aria-label={collapsed ? "Expand widget" : "Collapse widget"}
          onClick={() => setCollapsed((value) => !value)}
        >
          <ToggleIcon size={14} />
        </button>
        {icon ? <span className="context-widget-icon">{icon}</span> : null}
        <span className={`context-widget-status-dot context-widget-status-${status ?? "neutral"}`} aria-hidden="true" />
        <strong className="context-widget-title">{title}</strong>
        {collapsed && foldedHint ? (
          <small className="context-widget-folded-hint">{foldedHint}</small>
        ) : null}
        {badge != null ? (
          <span className={`context-widget-badge context-widget-badge-${badgeTone}`}>
            {badge}
          </span>
        ) : null}
        <div className="context-widget-header-spacer" />
        {actions ? <div className="context-widget-actions">{actions}</div> : null}
      </header>
      <div
        className={`context-widget-body ${scrollable || maxBodyHeight ? "context-widget-body-scrollable" : ""} ${bodyClassName}`}
        style={maxBodyHeight ? { "--context-widget-max-body-height": typeof maxBodyHeight === "number" ? `${maxBodyHeight}px` : maxBodyHeight } as CSSProperties : undefined}
      >
        {children}
      </div>
    </section>
  );
}
