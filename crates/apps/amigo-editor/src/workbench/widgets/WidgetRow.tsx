import type { ReactNode } from "react";
import type { WidgetBadgeTone } from "./widgetTypes";

export function WidgetRow({
  actions,
  badge,
  badgeTone = "neutral",
  className = "",
  icon,
  onClick,
  selected = false,
  subtitle,
  title,
}: {
  icon?: ReactNode;
  title: string;
  subtitle?: string;
  className?: string;
  badge?: ReactNode;
  badgeTone?: WidgetBadgeTone;
  actions?: ReactNode;
  selected?: boolean;
  onClick?: () => void;
}) {
  const content = (
    <>
      {icon ? <span className="workbench-widget-row-icon">{icon}</span> : null}
      <span className="workbench-widget-row-main">
        <strong>{title}</strong>
        {subtitle ? <small>{subtitle}</small> : null}
      </span>
      {badge ? (
        <span className={`workbench-badge workbench-badge-${badgeTone}`}>
          {badge}
        </span>
      ) : null}
      {actions ? <span className="workbench-widget-row-actions">{actions}</span> : null}
    </>
  );

  if (onClick) {
    return (
      <button
        className={`workbench-widget-row ${selected ? "selected" : ""} ${className}`}
        type="button"
        onClick={onClick}
      >
        {content}
      </button>
    );
  }

  return (
    <div className={`workbench-widget-row ${selected ? "selected" : ""} ${className}`}>
      {content}
    </div>
  );
}
