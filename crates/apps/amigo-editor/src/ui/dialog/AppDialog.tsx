import type { ReactNode } from "react";
import { X } from "lucide-react";

export function AppDialog({
  title,
  subtitle,
  icon,
  onClose,
  closeDisabled = false,
  backdropClassName = "new-project-backdrop",
  dialogClassName = "new-project-dialog",
  headerClassName = "new-project-header",
  bodyClassName = "new-project-body",
  footer,
  children,
}: {
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  onClose: () => void;
  closeDisabled?: boolean;
  backdropClassName?: string;
  dialogClassName?: string;
  headerClassName?: string;
  bodyClassName?: string;
  footer?: ReactNode;
  children: ReactNode;
}) {
  return (
    <div className={backdropClassName} onMouseDown={closeDisabled ? undefined : onClose}>
      <section
        className={dialogClassName}
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className={headerClassName}>
          <div className="app-dialog-header-main">
            {icon ? <span className="app-dialog-header-icon" aria-hidden="true">{icon}</span> : null}
            <h2>{title}</h2>
            {subtitle ? <p>{subtitle}</p> : null}
          </div>
          <button
            className="new-project-close-button"
            type="button"
            aria-label="Close dialog"
            onClick={onClose}
            disabled={closeDisabled}
          >
            <X size={16} />
          </button>
        </header>
        <div className={bodyClassName}>{children}</div>
        {footer ? <footer className="new-project-footer">{footer}</footer> : null}
      </section>
    </div>
  );
}
