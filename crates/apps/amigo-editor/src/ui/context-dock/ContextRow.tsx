import type { ReactNode } from "react";
import type { ContextRowTone } from "./contextDockTypes";

export function ContextRow({
  actions,
  badge,
  className = "",
  icon,
  onClick,
  selected = false,
  subtitle,
  title,
  tone = "default",
}: {
  icon?: ReactNode;
  title: string;
  subtitle?: string;
  className?: string;
  badge?: ReactNode;
  actions?: ReactNode;
  selected?: boolean;
  tone?: ContextRowTone;
  onClick?: () => void;
}) {
  const content = (
    <>
      {icon ? (
        <span className={`context-row-icon context-row-icon-${tone}`}>
          {icon}
        </span>
      ) : null}
      <span className="context-row-main">
        <strong>{title}</strong>
        {subtitle ? <small>{subtitle}</small> : null}
      </span>
      {badge ? <span className="context-row-badge">{badge}</span> : null}
      {actions ? <span className="context-row-actions">{actions}</span> : null}
    </>
  );

  if (onClick) {
    return (
      <button className={`context-row ${selected ? "selected" : ""} ${className}`} type="button" onClick={onClick}>
        {content}
      </button>
    );
  }

  return (
    <div className={`context-row ${selected ? "selected" : ""} ${className}`}>
      {content}
    </div>
  );
}

export function ContextMiniAction({
  children,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button className="context-mini-action" type="button" {...props}>
      {children}
    </button>
  );
}
