import type { EditorProjectFileDto } from "../../../../api/dto";
import { FileCode2, FileText, FolderOpen } from "lucide-react";
import { ContextActionStrip } from "../../../../ui/context-dock/ContextActionStrip";
import { ContextRow } from "../../../../ui/context-dock/ContextRow";
import type { ContextAction } from "../../../../ui/context-dock/contextDockTypes";
import type { SceneContextModel } from "../sceneContextTypes";

export function SceneFileInfoTab({
  model,
  onOpenScript,
  onReveal,
  onShowYaml,
  scriptFile,
  yamlFile,
}: {
  model: SceneContextModel;
  yamlFile: EditorProjectFileDto | null;
  scriptFile: EditorProjectFileDto | null;
  onShowYaml: () => void;
  onOpenScript: () => void;
  onReveal: () => void;
}) {
  const actions: ContextAction[] = [
    {
      id: "open-yaml",
      label: "Open YAML",
      icon: <FileText size={13} />,
      disabled: !yamlFile,
      onClick: onShowYaml,
    },
    {
      id: "open-script",
      label: "Open Script",
      icon: <FileCode2 size={13} />,
      disabled: !scriptFile,
      onClick: onOpenScript,
    },
    {
      id: "reveal-file",
      label: "Reveal",
      icon: <FolderOpen size={13} />,
      disabled: !yamlFile,
      onClick: onReveal,
    },
  ];

  return (
    <div className="scene-detail-tab">
      <section className="scene-detail-section">
      <ContextActionStrip actions={actions} />
      <ContextRow title="YAML" subtitle={yamlFile?.relativePath ?? model.fileInfo.source.yaml?.path ?? "missing"} />
      <ContextRow title="Script" subtitle={scriptFile?.relativePath ?? model.fileInfo.source.script?.path ?? "none"} />
      <ContextRow title="Folder" subtitle={model.fileInfo.source.folderPath ?? "unknown"} />
      </section>
    </div>
  );
}
