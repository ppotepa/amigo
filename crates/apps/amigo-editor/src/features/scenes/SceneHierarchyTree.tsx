import { Box, FileCode2, FileText, LayoutPanelTop, Monitor, ScanEye } from "lucide-react";
import type {
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";
import { UiNodeKindIcon } from "../../editors/ui-document/uiNodeKindIcons";
import {
  projectFilePathToTarget,
  sceneEntityToTarget,
  sceneToTarget,
  type EditorTargetIntent,
  type EditorTargetRef,
  uiDocumentToTarget,
  uiNodeDtoToTarget,
} from "../../editor-targets";
import { TreeView, useTreeExpansion, type TreeNodeAdapter, type TreeNodeCapabilities } from "../../ui/tree";

export type SceneTreeNode =
  | { kind: "scene"; id: string; label: string; detail: string; scene: EditorSceneSummaryDto; children: SceneTreeNode[] }
  | { kind: "scene-document"; id: string; label: string; detail: string; scene: EditorSceneSummaryDto; children: SceneTreeNode[] }
  | { kind: "scene-script"; id: string; label: string; detail: string; scene: EditorSceneSummaryDto; children: SceneTreeNode[] }
  | { kind: "entity"; id: string; label: string; scene: EditorSceneSummaryDto; entity: EditorSceneEntityDto; children: SceneTreeNode[] }
  | { kind: "ui-document"; id: string; label: string; scene: EditorSceneSummaryDto; document: EditorUiDocumentDto; children: SceneTreeNode[] }
  | { kind: "ui-node"; id: string; label: string; scene: EditorSceneSummaryDto; document: EditorUiDocumentDto; node: EditorUiNodeDto; children: SceneTreeNode[] };

function sceneTreeCapabilities(node: SceneTreeNode, context: { hasChildren: boolean }): TreeNodeCapabilities {
  return {
    canExpand: context.hasChildren,
    canSelect: true,
    canOpen: true,
    canAddChild: false,
    canRename: false,
    canDelete: false,
    canDrag: false,
    canDropOn: false,
  };
}

// @codemap anchor:scene-hierarchy-tree-adapter domain:scene-editor role:tree-adapter priority:P1 layer:app tags:tree,scene,adapter,editor-target
const sceneHierarchyTreeAdapter: TreeNodeAdapter<SceneTreeNode> = {
  getId: (node) => node.id,
  getLabel: (node) => node.label,
  getChildren: (node) => node.children,
  getIcon: (node) => <SceneTreeIcon node={node} />,
  getMeta: (node) => {
    if (node.kind === "entity") return node.entity.componentTypes[0] ?? "entity";
    if (node.kind === "scene") return node.scene.status;
    if (node.kind === "ui-node") return node.node.kind;
    if ("detail" in node) return node.detail;
    return null;
  },
  getBadges: (node) => [
    {
      label: "hidden",
      tone: "muted",
      visible: node.kind === "entity" && !node.entity.visible,
    },
    {
      label: node.kind === "ui-node" ? String(node.node.childCount || node.node.kind) : "",
      tone: node.kind === "ui-node" && node.node.actionEvent ? "valid" : "muted",
      visible: node.kind === "ui-node",
    },
  ],
  getSubItems: (node) => {
    if (node.kind === "entity") {
      return [
        { key: "components", label: `${node.entity.componentCount} components` },
        {
          key: "tags",
          label: node.entity.tags.length ? `#${node.entity.tags.join(" #")}` : "",
          visible: node.entity.tags.length > 0,
        },
      ];
    }

    if (node.kind === "ui-node") {
      return [
        { key: "text", label: node.node.text ?? "", visible: Boolean(node.node.text) },
        { key: "action", label: node.node.actionEvent ?? "", visible: Boolean(node.node.actionEvent) },
      ];
    }

    return [];
  },
  getClassName: (node) => `scene-tree-node-${node.kind}`,
  getCapabilities: (node, context) => sceneTreeCapabilities(node, context),
};

// @codemap anchor:scene-hierarchy-target-tree domain:scene-editor role:tree priority:P1 layer:app tags:tree,scene,editor-target
export function SceneHierarchyTree({
  hierarchy,
  onActivateTarget,
  selectedEntityId,
  selectedScene,
  selectedUiNodeComponentIndex,
  selectedUiNodeEntityId,
  selectedUiNodePath,
}: {
  hierarchy?: EditorSceneHierarchyDto;
  onActivateTarget: (target: EditorTargetRef, intent: EditorTargetIntent) => void;
  selectedEntityId: string | null;
  selectedScene: EditorSceneSummaryDto;
  selectedUiNodeComponentIndex: number | null;
  selectedUiNodeEntityId: string | null;
  selectedUiNodePath: string | null;
}) {
  const nodes = buildSceneTree(selectedScene, hierarchy);
  const selectedId =
    selectedUiNodeEntityId && selectedUiNodeComponentIndex != null && selectedUiNodePath
      ? uiNodeId(selectedUiNodeEntityId, selectedUiNodeComponentIndex, selectedUiNodePath)
      : selectedEntityId ?? `scene:${selectedScene.id}`;
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: sceneHierarchyTreeAdapter,
    nodes,
    selectedId,
  });

  return (
    <TreeView
      actions={{
        onOpen: (node) => onActivateTarget(sceneTreeNodeToTarget(node), "open"),
        onSelect: (node) => onActivateTarget(sceneTreeNodeToTarget(node), "select"),
      }}
      adapter={sceneHierarchyTreeAdapter}
      className="scene-hierarchy-tree"
      expandedIds={expandedIds}
      nodes={nodes}
      onToggle={toggleExpanded}
      preset="outline"
      selectedId={selectedId}
    />
  );
}

// @codemap anchor:scene-hierarchy-target-mapper domain:scene-editor role:tree-adapter priority:P1 layer:app tags:scene,editor-target
function sceneTreeNodeToTarget(node: SceneTreeNode): EditorTargetRef {
  if (node.kind === "scene") return sceneToTarget(node.scene);
  if (node.kind === "scene-document") return projectFilePathToTarget(node.scene.documentPath);
  if (node.kind === "scene-script") return projectFilePathToTarget(node.scene.scriptPath);
  if (node.kind === "entity") return sceneEntityToTarget({ sceneId: node.scene.id, entity: node.entity });
  if (node.kind === "ui-document") return uiDocumentToTarget({ sceneId: node.scene.id, document: node.document });
  return uiNodeDtoToTarget({ sceneId: node.scene.id, document: node.document, node: node.node });
}

// @codemap anchor:scene-hierarchy-tree-builder domain:scene-editor role:model priority:P1 layer:app tags:tree,scene,ui-document
function buildSceneTree(
  selectedScene: EditorSceneSummaryDto,
  hierarchy: EditorSceneHierarchyDto | undefined,
): SceneTreeNode[] {
  const entityNodes =
    hierarchy?.entities.map((entity) => {
      const documents = hierarchy.uiDocuments.filter((document) => document.entityId === entity.id);
      return {
        kind: "entity" as const,
        id: entity.id,
        label: entity.name,
        scene: selectedScene,
        entity,
        children: documents.map((document) => uiDocumentNode(selectedScene, document)),
      };
    }) ?? [];

  return [
    {
      kind: "scene",
      id: `scene:${selectedScene.id}`,
      label: selectedScene.label,
      detail: selectedScene.id,
      scene: selectedScene,
      children: [
        {
          kind: "scene-document",
          id: `scene-document:${selectedScene.id}`,
          label: "Document",
          detail: selectedScene.documentPath,
          scene: selectedScene,
          children: [],
        },
        {
          kind: "scene-script",
          id: `scene-script:${selectedScene.id}`,
          label: "Script",
          detail: selectedScene.scriptPath,
          scene: selectedScene,
          children: [],
        },
        ...entityNodes,
      ],
    },
  ];
}

function uiDocumentNode(scene: EditorSceneSummaryDto, document: EditorUiDocumentDto): SceneTreeNode {
  return {
    kind: "ui-document",
    id: uiDocumentId(document),
    label: `${document.entityName} UI`,
    scene,
    document,
    children: [uiNodeTreeNode(scene, document, document.root)],
  };
}

function uiNodeTreeNode(scene: EditorSceneSummaryDto, document: EditorUiDocumentDto, node: EditorUiNodeDto): SceneTreeNode {
  return {
    kind: "ui-node",
    id: uiNodeId(document.entityId, document.componentIndex, node.path),
    label: node.label,
    scene,
    document,
    node,
    children: node.children.map((child) => uiNodeTreeNode(scene, document, child)),
  };
}

function uiDocumentId(document: EditorUiDocumentDto): string {
  return `ui-document:${document.entityId}:${document.componentIndex}`;
}

function uiNodeId(entityId: string, componentIndex: number, path: string): string {
  return `ui-node:${entityId}:${componentIndex}:${path}`;
}

function SceneTreeIcon({ node }: { node: SceneTreeNode }) {
  if (node.kind === "scene") return <Monitor size={14} />;
  if (node.kind === "scene-document") return <FileText size={14} />;
  if (node.kind === "scene-script") return <FileCode2 size={14} />;
  if (node.kind === "entity") return <Box size={14} />;
  if (node.kind === "ui-document") return <LayoutPanelTop size={14} />;
  if (node.kind === "ui-node") return <UiNodeKindIcon kind={node.node.kind} />;
  return <ScanEye size={14} />;
}
