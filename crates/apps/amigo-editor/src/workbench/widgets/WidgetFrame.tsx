import type { CSSProperties, ReactNode } from "react";
import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";
import type { WidgetStatus } from "./widgetTypes";

export type WidgetFrameProps = {
  id: string;
  title: string;
  status?: WidgetStatus;
  icon?: ReactNode;
  badge?: ReactNode;
  badgeTone?: WidgetStatus | "muted";
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
      className={`workbench-widget workbench-widget-frame ${compact ? "compact" : ""} ${collapsed ? "folded collapsed" : ""} ${className}`}
      data-widget-id={id}
    >
      <header
        className={`workbench-widget-header ${compact ? "workbench-widget-header-compact" : ""}`}
      >
        <button
          className="workbench-widget-toggle"
          type="button"
          title={collapsed ? "Expand" : "Collapse"}
          aria-label={collapsed ? "Expand widget" : "Collapse widget"}
          onClick={() => setCollapsed((value) => !value)}
        >
          <ToggleIcon size={14} />
        </button>
        {icon ? <span className="workbench-widget-icon">{icon}</span> : null}
        <span className={`workbench-widget-status-dot workbench-widget-status-${status ?? "neutral"}`} aria-hidden="true" />
        <strong className="workbench-widget-title">{title}</strong>
        {collapsed && foldedHint ? (
          <small className="workbench-widget-folded-hint">{foldedHint}</small>
        ) : null}
        {badge != null ? (
          <span className={`workbench-widget-badge workbench-widget-badge-${badgeTone}`}>
            {badge}
          </span>
        ) : null}
        <div className="workbench-widget-header-spacer" />
        {actions ? <div className="workbench-widget-actions">{actions}</div> : null}
      </header>
      <div
        className={`workbench-widget-body ${scrollable || maxBodyHeight ? "workbench-widget-body-scrollable" : ""} ${bodyClassName}`}
        style={maxBodyHeight ? { "--workbench-widget-max-body-height": typeof maxBodyHeight === "number" ? `${maxBodyHeight}px` : maxBodyHeight } as CSSProperties : undefined}
      >
        {children}
      </div>
    </section>
  );
}
