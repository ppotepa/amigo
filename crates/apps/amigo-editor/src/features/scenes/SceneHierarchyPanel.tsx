import type {
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
  EditorUiDocumentDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type {
  WorkspaceRuntimeServices,
  WorkspaceUiNodeSelectionRef,
} from "../../main-window/workspaceRuntimeServices";
import { SceneHierarchyTree } from "./SceneHierarchyTree";

// @codemap anchor:scene-hierarchy-panel domain:scene-editor role:tree priority:P1 layer:app tags:tree,scene,shared-tree
export function SceneHierarchyPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <SceneHierarchy
      hierarchy={services.hierarchy}
      loading={services.hierarchyTask?.status === "running"}
      onOpenUiDocumentEditor={(document) =>
        services.openUiDocumentEditor?.({
          sceneId: services.selectedScene?.id ?? "",
          entityId: document.entityId,
          componentIndex: document.componentIndex,
          titleOverride: `${document.entityName} UI`,
        })
      }
      onSelectEntity={(entityId) => services.selectSceneEntity?.(entityId)}
      onSelectUiNode={(selection) => services.selectUiNode?.(selection)}
      selectedEntityId={services.selectedEntity?.id ?? null}
      selectedScene={services.selectedScene ?? null}
      selectedUiNodeComponentIndex={services.selection?.kind === "uiNode" ? services.selection.nodeRef.componentIndex : null}
      selectedUiNodeEntityId={services.selection?.kind === "uiNode" ? services.selection.entity.id : null}
      selectedUiNodePath={services.selectedUiNode?.path ?? null}
    />
  );
}

function SceneHierarchy({
  hierarchy,
  loading,
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
  loading: boolean;
  onOpenUiDocumentEditor: (document: EditorUiDocumentDto) => void;
  onSelectEntity: (entityId: string) => void;
  onSelectUiNode: (selection: WorkspaceUiNodeSelectionRef) => void;
  selectedEntityId: string | null;
  selectedScene: EditorSceneSummaryDto | null;
  selectedUiNodeComponentIndex: number | null;
  selectedUiNodeEntityId: string | null;
  selectedUiNodePath: string | null;
}) {
  if (!selectedScene) {
    return <p className="muted workspace-empty">No scene selected.</p>;
  }

  return (
    <div className="dock-scroll">
      {loading ? <p className="muted workspace-note">Indexing scene entities...</p> : null}
      <SceneHierarchyTree
        hierarchy={hierarchy}
        onOpenUiDocumentEditor={onOpenUiDocumentEditor}
        onSelectEntity={onSelectEntity}
        onSelectUiNode={onSelectUiNode}
        selectedEntityId={selectedEntityId}
        selectedScene={selectedScene}
        selectedUiNodeComponentIndex={selectedUiNodeComponentIndex}
        selectedUiNodeEntityId={selectedUiNodeEntityId}
        selectedUiNodePath={selectedUiNodePath}
      />
    </div>
  );
}
