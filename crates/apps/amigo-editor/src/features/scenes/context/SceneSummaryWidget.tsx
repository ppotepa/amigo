import { FileCode2, FileText, FolderOpen } from "lucide-react";
import type { EditorSceneSummaryDto } from "../../../api/dto";
import { ContextActionStrip } from "../../../ui/context-dock/ContextActionStrip";
import { ContextRow } from "../../../ui/context-dock/ContextRow";
import { ContextWidget } from "../../../ui/context-dock/ContextWidget";
import type { ContextAction } from "../../../ui/context-dock/contextDockTypes";
import { sceneContextIcon } from "./sceneContextIcons";

export function SceneSummaryWidget({
  onOpenScript,
  onReveal,
  onShowYaml,
  scene,
}: {
  scene: EditorSceneSummaryDto;
  onShowYaml: () => void;
  onOpenScript: () => void;
  onReveal?: () => void;
}) {
  const actions: ContextAction[] = [
    {
      id: "yaml",
      label: "YAML",
      icon: <FileText size={13} />,
      onClick: onShowYaml,
    },
    {
      id: "script",
      label: "Script",
      icon: <FileCode2 size={13} />,
      disabled: !scene.scriptPath,
      onClick: onOpenScript,
    },
    {
      id: "reveal",
      label: "Reveal",
      icon: <FolderOpen size={13} />,
      disabled: !onReveal,
      onClick: () => onReveal?.(),
    },
  ];

  return (
    <ContextWidget
      id="scene-summary"
      title="Scene Summary"
      icon={sceneContextIcon("scene")}
      badge={scene.status}
      badgeTone={scene.status === "valid" ? "valid" : "warning"}
    >
      <ContextRow
        icon={sceneContextIcon("scene")}
        title={scene.label}
        subtitle={scene.id}
        tone="cyan"
      />
      <ContextRow
        icon={<FileText size={13} />}
        title="Document"
        subtitle={scene.documentPath}
        badge="yaml"
      />
      <ContextActionStrip actions={actions} />
    </ContextWidget>
  );
}
