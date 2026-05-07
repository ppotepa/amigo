import { Box, FileCode2, FileText, LayoutPanelTop, Monitor, ScanEye } from "lucide-react";
import type {
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";
import { UiNodeKindIcon } from "../../editors/ui-document/uiNodeKindIcons";
import type { WorkspaceUiNodeSelectionRef } from "../../main-window/workspaceRuntimeServices";
import { TreeView, useTreeExpansion, type TreeNodeAdapter, type TreeNodeCapabilities } from "../../ui/tree";

type SceneTreeNode =
  | { kind: "scene"; id: string; label: string; detail: string; scene: EditorSceneSummaryDto; children: SceneTreeNode[] }
  | { kind: "scene-document"; id: string; label: string; detail: string; children: SceneTreeNode[] }
  | { kind: "scene-script"; id: string; label: string; detail: string; children: SceneTreeNode[] }
  | { kind: "entity"; id: string; label: string; entity: EditorSceneEntityDto; children: SceneTreeNode[] }
  | { kind: "ui-document"; id: string; label: string; document: EditorUiDocumentDto; children: SceneTreeNode[] }
  | { kind: "ui-node"; id: string; label: string; document: EditorUiDocumentDto; node: EditorUiNodeDto; children: SceneTreeNode[] };

function sceneTreeCapabilities(node: SceneTreeNode, context: { hasChildren: boolean }): TreeNodeCapabilities {
  return {
    canExpand: context.hasChildren,
    canSelect: node.kind === "entity" || node.kind === "ui-node",
    canOpen: node.kind === "ui-document" || node.kind === "ui-node",
    canAddChild: false,
    canRename: false,
    canDelete: false,
    canDrag: false,
    canDropOn: false,
  };
}

// @codemap anchor:scene-hierarchy-tree-adapter domain:scene-editor role:tree-adapter priority:P1 layer:app tags:tree,scene,adapter
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

export function SceneHierarchyTree({
  hierarchy,
  onOpenUiDocumentEditor,
  onSelectEntity,
  onSelectUiNode,
  selectedEntityId,
  selectedScene,
  selectedUiNodeComponentIndex,
  selectedUiNodeEntityId,
  selectedUiNodePath,
}: {
  hierarchy?: EditorSceneHierarchyDto;
  onOpenUiDocumentEditor: (document: EditorUiDocumentDto) => void;
  onSelectEntity: (entityId: string) => void;
  onSelectUiNode: (selection: WorkspaceUiNodeSelectionRef) => void;
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
      : selectedEntityId;
  const { expandedIds, toggleExpanded } = useTreeExpansion({
    adapter: sceneHierarchyTreeAdapter,
    nodes,
    selectedId,
  });

  return (
    <TreeView
      actions={{
        onOpen: (node) => {
          if (node.kind === "ui-document") onOpenUiDocumentEditor(node.document);
          if (node.kind === "ui-node") {
            onSelectUiNode({
              entityId: node.document.entityId,
              componentIndex: node.document.componentIndex,
              nodePath: node.node.path,
            });
            onOpenUiDocumentEditor(node.document);
          }
        },
        onSelect: (node) => {
          if (node.kind === "entity") onSelectEntity(node.entity.id);
          if (node.kind === "ui-node") {
            onSelectUiNode({
              entityId: node.document.entityId,
              componentIndex: node.document.componentIndex,
              nodePath: node.node.path,
            });
          }
        },
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
        entity,
        children: documents.map((document) => uiDocumentNode(document)),
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
          children: [],
        },
        {
          kind: "scene-script",
          id: `scene-script:${selectedScene.id}`,
          label: "Script",
          detail: selectedScene.scriptPath,
          children: [],
        },
        ...entityNodes,
      ],
    },
  ];
}

function uiDocumentNode(document: EditorUiDocumentDto): SceneTreeNode {
  return {
    kind: "ui-document",
    id: uiDocumentId(document),
    label: `${document.entityName} UI`,
    document,
    children: [uiNodeTreeNode(document, document.root)],
  };
}

function uiNodeTreeNode(document: EditorUiDocumentDto, node: EditorUiNodeDto): SceneTreeNode {
  return {
    kind: "ui-node",
    id: uiNodeId(document.entityId, document.componentIndex, node.path),
    label: node.label,
    document,
    node,
    children: node.children.map((child) => uiNodeTreeNode(document, child)),
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
