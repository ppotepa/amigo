import { Info } from "lucide-react";

export function OperationScopeHint({
  kind,
}: {
  kind: "project-item" | "scene-document";
}) {
  return (
    <div className={`operation-scope-hint ${kind}`}>
      <Info size={14} />
      {kind === "project-item" ? (
        <span>
          Project item changes are applied immediately. Save/Discard applies only to active scene document edits.
        </span>
      ) : (
        <span>
          Scene document changes use the active editor session. Use Save or Discard to commit or revert.
        </span>
      )}
    </div>
  );
}
