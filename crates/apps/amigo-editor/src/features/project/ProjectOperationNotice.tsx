import { CheckCircle2, XCircle } from "lucide-react";

export type ProjectOperationNoticeValue =
  | {
      tone: "success";
      message: string;
      detail?: string;
      primaryActionLabel?: string;
      onPrimaryAction?: () => void;
    }
  | {
      tone: "error";
      message: string;
      detail?: string;
    };

export function ProjectOperationNotice({
  notice,
  onDismiss,
}: {
  notice: ProjectOperationNoticeValue | null;
  onDismiss: () => void;
}) {
  if (!notice) return null;

  const Icon = notice.tone === "success" ? CheckCircle2 : XCircle;

  return (
    <div className={`project-operation-notice ${notice.tone}`}>
      <Icon size={15} />
      <span>
        <strong>{notice.message}</strong>
        {notice.detail ? <small>{notice.detail}</small> : null}
      </span>

      {"primaryActionLabel" in notice && notice.primaryActionLabel && notice.onPrimaryAction ? (
        <button className="button button-ghost" type="button" onClick={notice.onPrimaryAction}>
          {notice.primaryActionLabel}
        </button>
      ) : null}

      <button className="button button-ghost" type="button" onClick={onDismiss}>
        Dismiss
      </button>
    </div>
  );
}
