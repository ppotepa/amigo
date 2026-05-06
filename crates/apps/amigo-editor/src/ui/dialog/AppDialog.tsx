import type { ReactNode } from "react";
import { X } from "lucide-react";

function cx(...classes: Array<string | false | null | undefined>): string {
  return classes.filter(Boolean).join(" ");
}

export function AppDialog({
  title,
  titleTag,
  subtitle,
  icon,
  headerVariant = "standard",
  iconClassName,
  toneClassName,
  mode = "modal",
  onClose,
  closeDisabled = false,
  backdropClassName,
  dialogClassName,
  headerClassName,
  bodyClassName,
  footerClassName,
  footer,
  children,
}: {
  title: string;
  titleTag?: string;
  subtitle?: string;
  icon?: ReactNode;
  iconClassName?: string;
  toneClassName?: string;
  mode?: "modal" | "inline";
  headerVariant?: "standard" | "windows";
  onClose?: () => void;
  closeDisabled?: boolean;
  backdropClassName?: string;
  dialogClassName?: string;
  headerClassName?: string;
  bodyClassName?: string;
  footerClassName?: string;
  footer?: ReactNode;
  children: ReactNode;
}) {
  const dialog = (
    <section
      className={cx("app-dialog", toneClassName, dialogClassName)}
      role="dialog"
      aria-modal={mode === "modal"}
      onMouseDown={(event) => event.stopPropagation()}
    >
      <header className={cx("app-dialog-header", headerClassName)}>
        <div className={`app-dialog-header-main ${icon ? "has-icon" : "no-icon"}`}>
          {icon ? (
            <span className={`app-dialog-header-icon ${iconClassName ?? ""}`} aria-hidden="true">
              {icon}
            </span>
          ) : null}
          {headerVariant === "windows" ? (
            <h2 className="app-dialog-windows-title">
              <span className="app-dialog-windows-main">{title}</span>
              {titleTag ? <span className="app-dialog-windows-tag"> : {titleTag}</span> : null}
            </h2>
          ) : (
            <h2>{title}</h2>
          )}
          {subtitle ? <p>{subtitle}</p> : null}
        </div>
        {onClose ? (
          <button
            className="app-dialog-close-button"
            type="button"
            aria-label="Close dialog"
            onClick={onClose}
            disabled={closeDisabled}
          >
            <X size={16} />
          </button>
        ) : null}
      </header>
      <div className={cx("app-dialog-body", bodyClassName)}>{children}</div>
      {footer ? <footer className={cx("app-dialog-footer", footerClassName)}>{footer}</footer> : null}
    </section>
  );

  if (mode === "inline") {
    return dialog;
  }

  return (
    <div className={cx("app-dialog-backdrop", backdropClassName)} onMouseDown={closeDisabled || !onClose ? undefined : onClose}>
      {dialog}
    </div>
  );
}
