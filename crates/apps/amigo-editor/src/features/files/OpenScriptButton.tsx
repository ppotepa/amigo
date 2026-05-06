import { FileCode2 } from "lucide-react";
import type { ScriptSourceRef } from "./scriptSourceRefs";

export function OpenScriptButton({
  onOpen,
  source,
}: {
  source: ScriptSourceRef | null | undefined;
  onOpen?: (source: ScriptSourceRef) => void;
}) {
  if (!source) return null;

  return (
    <button
      className="button button-tool"
      type="button"
      title={source.title ?? source.path}
      disabled={!onOpen}
      onClick={() => onOpen?.(source)}
    >
      <FileCode2 size={14} />
      Script
    </button>
  );
}
