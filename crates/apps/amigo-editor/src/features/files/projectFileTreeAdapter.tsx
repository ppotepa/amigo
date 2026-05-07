import { Code2, FileText, Folder, Grid2X2, Image, Map, Music, Package, Type } from "lucide-react";
import type { EditorProjectFileDto } from "../../api/dto";
import { toneForFileKind } from "../../theme/semanticColorRegistry";
import type { TreeNodeAdapter, TreeNodeCapabilities } from "../../ui/tree";
import { fileIcon } from "./ProjectFileTree";

function fileCapabilities(file: EditorProjectFileDto, context: { hasChildren: boolean }): TreeNodeCapabilities {
  return {
    canExpand: file.isDir && context.hasChildren,
    canSelect: !file.isDir,
    canOpen: !file.isDir,
    canAddChild: false,
    canRename: false,
    canDelete: false,
    canDrag: false,
    canDropOn: file.isDir,
  };
}

// @codemap anchor:project-file-tree-adapter domain:project role:tree-adapter priority:P1 layer:app tags:tree,files,adapter
export const projectFileTreeAdapter: TreeNodeAdapter<EditorProjectFileDto> = {
  getId: (file) => projectFileTreeId(file),
  getLabel: (file) => file.name,
  getChildren: (file) => file.children ?? [],
  getIcon: (file) => <ProjectFileIcon file={file} />,
  getMeta: (file) => (file.isDir ? `${file.children?.length ?? 0}` : file.kind),
  getSubItems: (file) =>
    file.isDir
      ? []
      : [
          {
            key: "path",
            label: file.relativePath,
            title: file.relativePath,
          },
        ],
  getClassName: (file) =>
    file.isDir
      ? "project-file-tree-dir"
      : `project-file-tree-file ${toneForFileKind(file.kind || file.relativePath)}`,
  getCapabilities: (file, context) => fileCapabilities(file, context),
};

export function projectFileTreeId(file: Pick<EditorProjectFileDto, "relativePath" | "path">): string {
  return file.relativePath || file.path || "root";
}

function ProjectFileIcon({ file }: { file: EditorProjectFileDto }) {
  if (file.isDir) return <Folder size={14} />;

  switch (file.kind) {
    case "texture":
    case "rawImage":
      return <Image size={14} />;
    case "spritesheet":
      return <Grid2X2 size={14} />;
    case "audio":
      return <Music size={14} />;
    case "font":
      return <Type size={14} />;
    case "script":
    case "sceneScript":
      return <Code2 size={14} />;
    case "tilemap":
    case "tileset":
      return <Map size={14} />;
    case "scriptPackage":
      return <Package size={14} />;
    default:
      return <FileText size={14} aria-label={fileIcon(file)} />;
  }
}
