import { AlertTriangle, CheckCircle2, Info, X } from "lucide-react";

export type OperationNoticeTone = "success" | "error" | "info" | "warning";

export type OperationNoticeValue = {
  tone: OperationNoticeTone;
  message: string;
  detail?: string;
  primaryActionLabel?: string;
  onPrimaryAction?: () => void;
};

export function OperationNotice({
  notice,
  onDismiss,
}: {
  notice: OperationNoticeValue | null;
  onDismiss: () => void;
}) {
  if (!notice) {
    return null;
  }

  const Icon = iconForTone(notice.tone);

  return (
    <section className={`operation-notice operation-notice-${notice.tone}`}>
      <span className="operation-notice-icon">
        <Icon size={15} />
      </span>

      <span className="operation-notice-copy">
        <strong>{notice.message}</strong>
        {notice.detail ? <small>{notice.detail}</small> : null}
      </span>

      {notice.primaryActionLabel && notice.onPrimaryAction ? (
        <button className="button button-ghost button-compact" type="button" onClick={notice.onPrimaryAction}>
          {notice.primaryActionLabel}
        </button>
      ) : null}

      <button className="icon-button" type="button" aria-label="Dismiss notice" onClick={onDismiss}>
        <X size={14} />
      </button>
    </section>
  );
}

function iconForTone(tone: OperationNoticeTone) {
  switch (tone) {
    case "success":
      return CheckCircle2;
    case "error":
    case "warning":
      return AlertTriangle;
    case "info":
    default:
      return Info;
  }
}
