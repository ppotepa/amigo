import { FileCode2 } from "lucide-react";
import type { YamlSourceRef } from "./yamlSourceRefs";

export function ShowYamlButton({
  source,
  onShow,
}: {
  source: YamlSourceRef | null | undefined;
  onShow?: (source: YamlSourceRef) => void;
}) {
  if (!source) return null;

  return (
    <button
      className="button button-tool"
      type="button"
      title={source.title ?? source.path}
      disabled={!onShow}
      onClick={() => onShow?.(source)}
    >
      <FileCode2 size={14} />
      Show YAML View
    </button>
  );
}
