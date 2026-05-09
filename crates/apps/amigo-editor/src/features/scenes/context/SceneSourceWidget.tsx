import { FileCode2, FileText, FolderOpen } from "lucide-react";
import type { EditorProjectFileDto } from "../../../api/dto";
import { ContextRow } from "../../../ui/context-dock/ContextRow";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import { sceneContextIcon } from "./sceneContextIcons";
import type { SceneSourceModel } from "./sceneContextTypes";

export function SceneSourceWidget({
  onOpenScript,
  onReveal,
  onShowYaml,
  scriptFile,
  source,
  yamlFile,
}: {
  source: SceneSourceModel;
  yamlFile: EditorProjectFileDto | null;
  scriptFile: EditorProjectFileDto | null;
  onShowYaml: () => void;
  onOpenScript: () => void;
  onReveal?: () => void;
}) {
  return (
    <WidgetFrame
      id="scene-source"
      title="Source"
      icon={sceneContextIcon("scene")}
      badge={source.script ? 3 : 2}
      badgeTone="muted"
      defaultCollapsed
    >
      <ContextRow
        icon={<FileText size={13} />}
        title={yamlFile?.name ?? "scene.yml"}
        subtitle={source.yaml?.path ?? "Missing YAML source"}
        badge="yaml"
        tone={yamlFile ? "blue" : "orange"}
        onClick={yamlFile ? onShowYaml : undefined}
      />
      <ContextRow
        icon={<FileCode2 size={13} />}
        title={scriptFile?.name ?? "scene.rhai"}
        subtitle={source.script?.path ?? "No scene script"}
        badge="rhai"
        tone={scriptFile ? "green" : "orange"}
        onClick={scriptFile ? onOpenScript : undefined}
      />
      <ContextRow
        icon={<FolderOpen size={13} />}
        title="Reveal folder"
        subtitle={source.folderPath ?? "Scene folder"}
        badge="folder"
        tone="default"
        onClick={onReveal}
      />
    </WidgetFrame>
  );
}
