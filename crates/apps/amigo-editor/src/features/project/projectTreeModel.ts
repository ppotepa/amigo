import type { EditorProjectFileDto, EditorProjectStructureNodeDto, EditorSceneSummaryDto } from "../../api/dto";
import { normalizePath } from "../files/fileTreeSelectors";

export type ProjectTreeNodeStatus = "ok" | "valid" | "ready" | "warn" | "error" | "empty" | "missing" | "cached";

export type ProjectTreeNodeKind =
  | "modRoot"
  | "overview"
  | "manifest"
  | "folder"
  | "expectedFolder"
  | "scene"
  | "sceneDocument"
  | "sceneScript"
  | "assetCategory"
  | "assetResource"
  | "assetFile"
  | "scriptFile"
  | "scriptPackage"
  | "virtualGroup"
  | "capabilities"
  | "dependencies"
  | "diagnostics";

export interface ProjectTreeNode {
  id: string;
  label: string;
  kind: ProjectTreeNodeKind;
  icon: string;
  status?: ProjectTreeNodeStatus;
  count?: number;
  path?: string;
  expectedPath?: string;
  exists: boolean;
  empty?: boolean;
  ghost?: boolean;
  file?: EditorProjectFileDto;
  scene?: EditorSceneSummaryDto;
  children?: ProjectTreeNode[];
}

export type ProjectExplorerTreeNode = ProjectTreeNode | EditorProjectStructureNodeDto;

export function mergeProjectTrees(
  preferred: ProjectExplorerTreeNode,
  fallback: ProjectTreeNode,
): ProjectTreeNode {
  const fallbackChildrenById = new globalThis.Map((fallback.children ?? []).map((child) => [child.id, child]));
  const preferredChildren = preferred.children ?? [];
  const mergedChildren = preferredChildren.length > 0
    ? preferredChildren.map((child) => {
        const fallbackChild = fallbackChildrenById.get(child.id);
        if (!fallbackChild) return normalizeProjectTreeNode(child);
        if (fallbackChild.kind === "scene") {
          return normalizeProjectTreeNode(child, fallbackChild.children ?? []);
        }
        return mergeProjectTrees(child, fallbackChild);
      })
    : (fallback.children ?? []);

  return normalizeProjectTreeNode(preferred, mergedChildren);
}

export function normalizeProjectTreeNode(
  node: ProjectExplorerTreeNode,
  children?: ProjectTreeNode[],
): ProjectTreeNode {
  return {
    id: node.id,
    label: node.label,
    kind: node.kind as ProjectTreeNodeKind,
    icon: node.icon,
    status: (node.status ?? undefined) as ProjectTreeNodeStatus | undefined,
    count: node.count ?? undefined,
    path: node.path ?? undefined,
    expectedPath: node.expectedPath ?? undefined,
    exists: node.exists,
    empty: node.empty ?? false,
    ghost: node.ghost ?? false,
    file: node.file ?? undefined,
    scene: node.scene ?? undefined,
    children: children ?? (node.children ?? []).map((child) => normalizeProjectTreeNode(child)),
  };
}

export function projectNodeKindLabel(kind: string): string {
  return kind.replace(/([A-Z])/g, " $1").toLowerCase().trim();
}

export function projectNodeMatchesSearch(node: ProjectExplorerTreeNode, search: string): boolean {
  const ownText = `${node.label} ${node.kind} ${node.status ?? ""} ${node.path ?? ""} ${node.expectedPath ?? ""}`.toLowerCase();
  return ownText.includes(search) || (node.children ?? []).some((child) => projectNodeMatchesSearch(child, search));
}

export function relativeProjectPath(path: string): string {
  const normalized = normalizePath(path);
  for (const prefix of ["scenes/", "raw/", "spritesheets/", "audio/", "fonts/", "scripts/", "data/", "docs/", "custom/", "packages/"]) {
    const index = normalized.indexOf(prefix);
    if (index >= 0) return normalized.slice(index);
  }
  return normalized;
}

export function statusForEditorStatus(status: string): ProjectTreeNodeStatus {
  if (status === "valid") return "valid";
  if (status === "warning" || status === "missingDependency") return "warn";
  if (status === "error" || status === "invalidManifest" || status === "missingSceneFile" || status === "previewFailed") return "error";
  return "ok";
}

export function assetDisplayLabel(file: EditorProjectFileDto): string {
  return file.name.replace(
    /\.(image|sprite|atlas|tileset|tile-ruleset|tilemap|font|audio|particle|material|ui)\.ya?ml$/i,
    "",
  );
}
