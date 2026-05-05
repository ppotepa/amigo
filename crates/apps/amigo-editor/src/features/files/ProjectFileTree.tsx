import type { EditorProjectFileDto } from "../../api/dto";

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

export function ProjectFileTree({
  node,
  depth,
  selectedFilePath,
  collapsed,
  onSelectFile,
  onToggle,
}: {
  node: EditorProjectFileDto;
  depth: number;
  selectedFilePath: string | null;
  collapsed: Record<string, boolean>;
  onSelectFile: (file: EditorProjectFileDto) => void;
  onToggle: (path: string) => void;
}) {
  const nodeId = node.relativePath || node.path || "root";
  const isCollapsed = Boolean(collapsed[nodeId]);
  const children = node.children ?? [];

  if (depth === 0) {
    return (
      <>
        {children.map((child) => (
          <ProjectFileTree
            key={child.relativePath || child.path}
            node={child}
            depth={1}
            selectedFilePath={selectedFilePath}
            collapsed={collapsed}
            onSelectFile={onSelectFile}
            onToggle={onToggle}
          />
        ))}
      </>
    );
  }

  return (
    <>
      <button
        type="button"
        className={`workspace-row ${selectedFilePath === node.relativePath ? "selected" : ""}`}
        style={{ paddingLeft: 7 + depth * 12 }}
        onClick={() => {
          if (node.isDir) {
            onToggle(nodeId);
            return;
          }
          onSelectFile(node);
        }}
      >
        <span className={`dock-icon ${node.isDir ? "dock-icon-blue" : "dock-icon-cyan"}`}>{fileIcon(node)}</span>
        <span>
          <strong>{node.name}</strong>
          <small>{node.isDir ? `${children.length} entries` : node.relativePath}</small>
        </span>
        <em className="badge badge-muted">{node.isDir ? (isCollapsed ? "+" : "-") : node.kind}</em>
      </button>
      {node.isDir && !isCollapsed
        ? children.map((child) => (
            <ProjectFileTree
              key={child.relativePath || child.path}
              node={child}
              depth={depth + 1}
              selectedFilePath={selectedFilePath}
              collapsed={collapsed}
              onSelectFile={onSelectFile}
              onToggle={onToggle}
            />
          ))
        : null}
    </>
  );
}
