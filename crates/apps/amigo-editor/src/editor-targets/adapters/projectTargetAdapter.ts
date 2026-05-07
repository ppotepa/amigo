import type { EditorProjectStructureNodeDto } from "../../api/dto";
import type {
  ProjectExplorerTreeNode,
  ProjectTreeNode,
} from "../../features/project/projectTreeModel";
import { normalizeProjectTreeNode } from "../../features/project/projectTreeModel";
import type { EditorTargetRef } from "../editorTargetTypes";
import { projectFileToTarget } from "./fileTargetAdapter";
import { sceneToTarget } from "./sceneTargetAdapter";

// @codemap anchor:project-target-adapter domain:project role:tree-adapter priority:P1 layer:app tags:editor-target,project,selection
export function projectNodeToTarget(node: ProjectExplorerTreeNode): EditorTargetRef {
  const normalized = normalizeProjectTreeNode(node);

  if (normalized.scene) {
    return sceneToTarget(normalized.scene);
  }

  if (normalized.file && !normalized.file.isDir) {
    return projectFileToTarget(normalized.file);
  }

  if (normalized.kind === "capabilities") {
    return { kind: "capability", capabilityId: normalized.id };
  }

  if (normalized.kind === "dependencies") {
    return { kind: "dependency", dependencyId: normalized.id };
  }

  return {
    kind: "projectNode",
    nodeId: normalized.id,
    nodeKind: normalized.kind,
    label: normalized.label,
    path: normalized.path ?? null,
    expectedPath: normalized.expectedPath ?? null,
  };
}

export function projectStructureNodeToTarget(node: EditorProjectStructureNodeDto): EditorTargetRef {
  return projectNodeToTarget(node);
}

export function projectTreeNodeToTarget(node: ProjectTreeNode): EditorTargetRef {
  return projectNodeToTarget(node);
}

export function modToTarget(modId: string): EditorTargetRef {
  return { kind: "mod", modId };
}
