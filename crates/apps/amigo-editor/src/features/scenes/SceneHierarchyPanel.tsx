import type {
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type {
  WorkspaceRuntimeServices,
  WorkspaceUiNodeSelectionRef,
} from "../../main-window/workspaceRuntimeServices";

export function SceneHierarchyPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <SceneHierarchy
      hierarchy={services.hierarchy}
      loading={services.hierarchyTask?.status === "running"}
      onSelectEntity={(entityId) => services.selectSceneEntity?.(entityId)}
      onSelectUiNode={(selection) => services.selectUiNode?.(selection)}
      selectedEntityId={services.selectedEntity?.id ?? null}
      selectedUiNodePath={services.selectedUiNode?.path ?? null}
      selectedUiNodeEntityId={services.selection?.kind === "uiNode" ? services.selection.entity.id : null}
      selectedUiNodeComponentIndex={services.selection?.kind === "uiNode" ? services.selection.nodeRef.componentIndex : null}
      selectedScene={services.selectedScene ?? null}
    />
  );
}

function SceneHierarchy({
  selectedScene,
  hierarchy,
  loading,
  selectedEntityId,
  selectedUiNodePath,
  selectedUiNodeEntityId,
  selectedUiNodeComponentIndex,
  onSelectEntity,
  onSelectUiNode,
}: {
  selectedScene: EditorSceneSummaryDto | null;
  hierarchy?: EditorSceneHierarchyDto;
  loading: boolean;
  selectedEntityId: string | null;
  selectedUiNodePath: string | null;
  selectedUiNodeEntityId: string | null;
  selectedUiNodeComponentIndex: number | null;
  onSelectEntity: (entityId: string) => void;
  onSelectUiNode: (selection: WorkspaceUiNodeSelectionRef) => void;
}) {
  if (!selectedScene) {
    return <p className="muted workspace-empty">No scene selected.</p>;
  }

  return (
    <div className="dock-scroll">
      <SectionTitle title="Scene Context" />
      <Row icon="Sc" title={selectedScene.label} detail={selectedScene.id} badge={selectedScene.status} selected />
      <Row icon="Y" title="Document" detail={selectedScene.documentPath} badge="yaml" />
      <Row icon="Rh" title="Script" detail={selectedScene.scriptPath} badge="rhai" />

      <SectionTitle title={`Entities ${hierarchy ? `(${hierarchy.entityCount})` : ""}`} />

      {loading ? (
        <p className="muted workspace-note">Indexing scene entities...</p>
      ) : hierarchy?.entities.length ? (
        hierarchy.entities.map((entity) => {
          const uiDocuments = hierarchy.uiDocuments.filter((document) => document.entityId === entity.id);

          return (
            <div key={entity.id} className="workspace-tree-group">
              <button
                type="button"
                className={`workspace-row ${entity.id === selectedEntityId ? "selected" : ""}`}
                onClick={() => onSelectEntity(entity.id)}
              >
                <span className="dock-icon dock-icon-blue">{entity.name.slice(0, 2).toUpperCase()}</span>
                <span>
                  <strong>{entity.name}</strong>
                  <small>
                    {entity.componentCount} components
                    {entity.tags.length ? ` · #${entity.tags.join(" #")}` : ""}
                  </small>
                </span>
                <em className={`badge ${entity.visible ? "badge-valid" : "badge-muted"}`}>
                  {entity.componentTypes[0] ?? "entity"}
                </em>
              </button>

              {uiDocuments.map((document) => (
                <UiDocumentTree
                  key={`${document.entityId}:${document.componentIndex}`}
                  document={document}
                  selectedPath={
                    selectedUiNodeEntityId === document.entityId &&
                    selectedUiNodeComponentIndex === document.componentIndex
                      ? selectedUiNodePath
                      : null
                  }
                  onSelectUiNode={onSelectUiNode}
                />
              ))}
            </div>
          );
        })
      ) : (
        <p className="muted workspace-note">No authored entities found in this scene document.</p>
      )}
    </div>
  );
}

function UiDocumentTree({
  document,
  selectedPath,
  onSelectUiNode,
}: {
  document: EditorUiDocumentDto;
  selectedPath: string | null;
  onSelectUiNode: (selection: WorkspaceUiNodeSelectionRef) => void;
}) {
  return (
    <div className="workspace-tree-nested">
      <button
        type="button"
        className="workspace-row workspace-row-compact"
        onClick={() =>
          onSelectUiNode({
            entityId: document.entityId,
            componentIndex: document.componentIndex,
            nodePath: document.root.path,
          })
        }
      >
        <span className="dock-icon dock-icon-purple">UI</span>
        <span>
          <strong>UiDocument #{document.componentIndex}</strong>
          <small>{document.targetLayer ?? "screen-space"}</small>
        </span>
        <em className="badge badge-muted">ui</em>
      </button>

      <UiNodeTree
        document={document}
        node={document.root}
        depth={0}
        selectedPath={selectedPath}
        onSelectUiNode={onSelectUiNode}
      />
    </div>
  );
}

function UiNodeTree({
  document,
  node,
  depth,
  selectedPath,
  onSelectUiNode,
}: {
  document: EditorUiDocumentDto;
  node: EditorUiNodeDto;
  depth: number;
  selectedPath: string | null;
  onSelectUiNode: (selection: WorkspaceUiNodeSelectionRef) => void;
}) {
  const selected = selectedPath === node.path;

  return (
    <div className="workspace-ui-node">
      <button
        type="button"
        className={`workspace-row workspace-row-compact ${selected ? "selected" : ""}`}
        style={{ paddingLeft: `${Math.min(14 + depth * 14, 56)}px` }}
        onClick={() =>
          onSelectUiNode({
            entityId: document.entityId,
            componentIndex: document.componentIndex,
            nodePath: node.path,
          })
        }
      >
        <span className="dock-icon dock-icon-cyan">{iconForUiKind(node.kind)}</span>
        <span>
          <strong>{node.label}</strong>
          <small>
            {node.kind}
            {node.text ? ` · ${node.text}` : ""}
            {node.actionEvent ? ` · ${node.actionEvent}` : ""}
          </small>
        </span>
        <em className={`badge ${node.actionEvent ? "badge-valid" : "badge-muted"}`}>
          {node.childCount ? `${node.childCount}` : node.kind}
        </em>
      </button>

      {node.children.map((child) => (
        <UiNodeTree
          key={child.path}
          document={document}
          node={child}
          depth={depth + 1}
          selectedPath={selectedPath}
          onSelectUiNode={onSelectUiNode}
        />
      ))}
    </div>
  );
}

function iconForUiKind(kind: string): string {
  switch (kind) {
    case "button":
      return "Bt";
    case "text":
      return "Tx";
    case "panel":
      return "Pn";
    case "column":
      return "Co";
    case "row":
      return "Ro";
    case "spacer":
      return "Sp";
    default:
      return "Ui";
  }
}

function SectionTitle({ title }: { title: string }) {
  return <div className="workspace-section-title">{title}</div>;
}

function Row({
  icon,
  title,
  detail,
  badge,
  selected,
}: {
  icon: string;
  title: string;
  detail: string;
  badge?: string;
  selected?: boolean;
}) {
  return (
    <div className={`workspace-row ${selected ? "selected" : ""}`}>
      <span className="dock-icon dock-icon-blue">{icon}</span>
      <span>
        <strong>{title}</strong>
        <small>{detail}</small>
      </span>
      {badge ? <em className="badge badge-muted">{badge}</em> : null}
    </div>
  );
}
