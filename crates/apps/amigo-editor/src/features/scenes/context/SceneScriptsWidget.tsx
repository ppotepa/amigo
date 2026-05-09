import { useMemo, useState } from "react";
import type React from "react";
import { FileCode2, Folder, Package } from "lucide-react";
import type { EditorProjectFileDto } from "../../../api/dto";
import { ContextSearch } from "../../../ui/context-dock/ContextSearch";
import { ContextTree } from "../../../ui/context-dock/ContextTree";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import type { ContextTreeNode } from "../../../ui/context-dock/contextDockTypes";
import { sceneContextIcon } from "./sceneContextIcons";
import type { SceneScriptRef } from "./sceneContextTypes";

const SCRIPT_ROLE_LABEL: Record<SceneScriptRef["role"], string> = {
  primary: "Primary",
  component: "Component",
  mod: "Mod",
  package: "Package",
  related: "Related",
};

const SCRIPT_GROUP_ORDER: SceneScriptRef["role"][] = [
  "primary",
  "component",
  "mod",
  "package",
  "related",
];

const SCRIPT_GROUP_LABEL: Record<SceneScriptRef["role"], string> = {
  primary: "Scene Entry",
  component: "Component Scripts",
  mod: "Mod Scripts",
  package: "Package Scripts",
  related: "Related Scripts",
};

const SCRIPT_GROUP_ICON: Record<SceneScriptRef["role"], React.ReactNode> = {
  primary: <FileCode2 size={13} />,
  component: sceneContextIcon("component"),
  mod: <Package size={13} />,
  package: <Package size={13} />,
  related: <Folder size={13} />,
};

export function SceneScriptsWidget({
  onOpenFile,
  scripts,
}: {
  scripts: SceneScriptRef[];
  onOpenFile?: (file: EditorProjectFileDto) => void;
}) {
  const [query, setQuery] = useState("");
  const filtered = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) return scripts;
    return scripts.filter((script) =>
      script.file.name.toLowerCase().includes(normalized) ||
      script.file.relativePath.toLowerCase().includes(normalized) ||
      script.role.includes(normalized),
    );
  }, [query, scripts]);

  const treeNodes = useMemo(
    () => scriptsToTree(filtered, onOpenFile),
    [filtered, onOpenFile],
  );

  return (
    <WidgetFrame
      id="scene-scripts"
      title="Scripts"
      icon={sceneContextIcon("script")}
      badge={scripts.length}
      badgeTone={scripts.length ? "info" : "muted"}
      maxBodyHeight={320}
    >
      <ContextSearch value={query} placeholder="Search scripts..." onChange={setQuery} />
      {treeNodes.length ? (
        <ContextTree nodes={treeNodes} />
      ) : (
        <p className="muted workspace-note">No scripts matched.</p>
      )}
    </WidgetFrame>
  );
}

function scriptsToTree(
  scripts: SceneScriptRef[],
  onOpenFile?: (file: EditorProjectFileDto) => void,
): ContextTreeNode[] {
  return SCRIPT_GROUP_ORDER
    .map((role): ContextTreeNode | null => {
      const roleScripts = scripts.filter((script) => script.role === role);
      if (!roleScripts.length) return null;

      const packageGroups = groupScriptsByFolder(roleScripts);

      return {
        id: `scripts:${role}`,
        title: SCRIPT_GROUP_LABEL[role],
        icon: SCRIPT_GROUP_ICON[role],
        badge: roleScripts.length,
        defaultExpanded: role === "primary" || role === "component",
        children: packageGroups.map(([folder, folderScripts]) => {
          if (folderScripts.length === 1 && folder === ".") {
            return scriptToNode(folderScripts[0], onOpenFile);
          }

          return {
            id: `scripts:${role}:${folder}`,
            title: folder,
            subtitle: `${folderScripts.length} files`,
            icon: <Folder size={13} />,
            badge: folderScripts.length,
            defaultExpanded: role === "primary" || role === "component",
            children: folderScripts.map((script) => scriptToNode(script, onOpenFile)),
          };
        }),
      };
    })
    .filter((node): node is ContextTreeNode => Boolean(node));
}

function scriptToNode(
  script: SceneScriptRef,
  onOpenFile?: (file: EditorProjectFileDto) => void,
): ContextTreeNode {
  return {
    id: script.id,
    title: script.file.name,
    subtitle: script.file.relativePath,
    icon: <FileCode2 size={13} />,
    badge: <span className={`badge script-role-${script.role}`}>{SCRIPT_ROLE_LABEL[script.role]}</span>,
    onSelect: () => onOpenFile?.(script.file),
  };
}

function groupScriptsByFolder(scripts: SceneScriptRef[]): Array<[string, SceneScriptRef[]]> {
  const groups = new Map<string, SceneScriptRef[]>();
  for (const script of scripts) {
    const folder = script.file.relativePath.split("/").slice(0, -1).join("/") || ".";
    const bucket = groups.get(folder) ?? [];
    bucket.push(script);
    groups.set(folder, bucket);
  }

  return Array.from(groups.entries())
    .map(([folder, group]) => [
      folder,
      group.sort((left, right) => left.file.name.localeCompare(right.file.name)),
    ] as [string, SceneScriptRef[]])
    .sort(([left], [right]) => left.localeCompare(right));
}
