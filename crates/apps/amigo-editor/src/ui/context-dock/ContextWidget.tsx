import { useState, type ReactNode } from "react";
import type React from "react";
import type { ContextBadgeTone } from "./contextDockTypes";
import { ContextWidgetHeader } from "./ContextWidgetHeader";

export function ContextWidget({
  actions,
  badge,
  badgeTone,
  bodyClassName = "",
  children,
  className = "",
  collapsed,
  defaultCollapsed = false,
  icon,
  id,
  maxBodyHeight,
  onCollapsedChange,
  scrollable = true,
  title,
}: {
  id: string;
  title: string;
  icon?: ReactNode;
  badge?: ReactNode;
  badgeTone?: ContextBadgeTone;
  actions?: ReactNode;
  children: ReactNode;
  bodyClassName?: string;
  className?: string;
  defaultCollapsed?: boolean;
  collapsed?: boolean;
  maxBodyHeight?: number | string;
  onCollapsedChange?: (collapsed: boolean) => void;
  scrollable?: boolean;
}) {
  const [internalCollapsed, setInternalCollapsed] = useState(defaultCollapsed);
  const actualCollapsed = collapsed ?? internalCollapsed;

  function toggle() {
    const next = !actualCollapsed;
    if (collapsed == null) {
      setInternalCollapsed(next);
    }
    onCollapsedChange?.(next);
  }

  return (
    <section
      className={`context-widget ${actualCollapsed ? "collapsed" : ""} ${className}`}
      data-context-widget-id={id}
    >
      <ContextWidgetHeader
        actions={actions}
        badge={badge}
        badgeTone={badgeTone}
        collapsed={actualCollapsed}
        icon={icon}
        title={title}
        onToggle={toggle}
      />
      {!actualCollapsed ? (
        <div
          className={`context-widget-body ${scrollable ? "context-widget-body-scrollable" : ""} ${bodyClassName}`}
          style={{
            "--context-widget-max-body-height": typeof maxBodyHeight === "number"
              ? `${maxBodyHeight}px`
              : maxBodyHeight,
          } as React.CSSProperties}
        >
          {children}
        </div>
      ) : null}
    </section>
  );
}
