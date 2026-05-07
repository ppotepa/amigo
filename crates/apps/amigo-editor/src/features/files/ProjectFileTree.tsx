import type { EditorProjectFileDto } from "../../api/dto";
import { TreeView, useTreeExpansion } from "../../ui/tree";
import { projectFileTreeAdapter, projectFileTreeId } from "./projectFileTreeAdapter";

export function fileIcon(file: EditorProjectFileDto): string {
  if (file.isDir) return "Dir";
  if (file.kind === "manifest") return "T";
  if (file.kind === "sceneDocument") return "Y";
  if (file.kind === "sceneScript") return "Rh";
  if (file.kind === "scriptPackage") return "Pkg";
  if (file.kind === "script") return "Rh";
  if (file.kind === "texture") return "Tx";
  if (file.kind === "spritesheet") return "Sp";
  if (file.kind === "audio") return "Au";
  if (file.kind === "font") return "Fn";
  if (file.kind === "tilemap") return "Tm";
  if (file.kind === "tileset") return "Ts";
  return "F";
}

// @codemap anchor:project-file-tree-panel domain:project role:tree priority:P1 layer:app tags:tree,files,shared-tree,editor-target
export function ProjectFileTree({
  node,
  selectedFilePath,
  onOpenFile,
  onSelectFile,
}: {
  node: EditorProjectFileDto;
  selectedFilePath: string | null;
  onOpenFile: (file: EditorProjectFileDto) => void;
  onSelectFile: (file: EditorProjectFileDto) => void;
}) {
  const selectedId = selectedFilePath ?? null;
  const nodes = [node];
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: projectFileTreeAdapter,
    nodes,
    selectedId,
  });

  return (
    <TreeView
      actions={{
        onOpen: (file) => {
          if (!file.isDir) onOpenFile(file);
        },
        onSelect: (file) => {
          if (!file.isDir) onSelectFile(file);
        },
      }}
      adapter={projectFileTreeAdapter}
      className="project-file-tree"
      expandedIds={expandedIds}
      nodes={nodes}
      onToggle={toggleExpanded}
      preset="explorer"
      selectedId={selectedId ? projectFileTreeId({ relativePath: selectedId, path: selectedId }) : null}
    />
  );
}
