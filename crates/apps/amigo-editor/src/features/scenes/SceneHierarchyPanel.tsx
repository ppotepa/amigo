import type {
  EditorSceneHierarchyDto,
  EditorSceneSummaryDto,
} from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { EditorTargetIntent, EditorTargetRef } from "../../editor-targets/editorTargetTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";
import { SceneHierarchyTree } from "./SceneHierarchyTree";

// @codemap anchor:scene-hierarchy-panel domain:scene-editor role:tree priority:P1 layer:app tags:tree,scene,shared-tree,editor-target
export function SceneHierarchyPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  const target = services.currentEditorTarget?.ref ?? null;
  const selectedEntityId =
    target?.kind === "sceneEntity"
      ? target.entityId
      : services.selectedEntity?.id ?? null;
  const selectedUiNodeEntityId =
    target?.kind === "uiNode"
      ? target.entityId
      : services.currentEditorTarget?.selection.kind === "uiNode"
        ? services.currentEditorTarget.selection.nodeRef.entityId
        : services.selection?.kind === "uiNode"
          ? services.selection.nodeRef.entityId
          : null;
  const selectedUiNodeComponentIndex =
    target?.kind === "uiNode"
      ? target.componentIndex
      : services.currentEditorTarget?.selection.kind === "uiNode"
        ? services.currentEditorTarget.selection.nodeRef.componentIndex
        : services.selection?.kind === "uiNode"
          ? services.selection.nodeRef.componentIndex
          : null;
  const selectedUiNodePath =
    target?.kind === "uiNode"
      ? target.nodePath
      : services.currentEditorTarget?.selection.kind === "uiNode"
        ? services.currentEditorTarget.selection.nodeRef.nodePath
        : services.selectedUiNode?.path ?? null;

  return (
    <SceneHierarchy
      hierarchy={services.hierarchy}
      loading={services.hierarchyTask?.status === "running"}
      onActivateTarget={(nextTarget, intent) => services.activateEditorTarget?.(nextTarget, intent)}
      selectedEntityId={selectedEntityId}
      selectedScene={services.selectedScene ?? null}
      selectedUiNodeComponentIndex={selectedUiNodeComponentIndex}
      selectedUiNodeEntityId={selectedUiNodeEntityId}
      selectedUiNodePath={selectedUiNodePath}
    />
  );
}

function SceneHierarchy({
  hierarchy,
  loading,
  onActivateTarget,
  selectedEntityId,
  selectedScene,
  selectedUiNodeComponentIndex,
  selectedUiNodeEntityId,
  selectedUiNodePath,
}: {
  hierarchy?: EditorSceneHierarchyDto;
  loading: boolean;
  onActivateTarget: (target: EditorTargetRef, intent: EditorTargetIntent) => void;
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
        onActivateTarget={onActivateTarget}
        selectedEntityId={selectedEntityId}
        selectedScene={selectedScene}
        selectedUiNodeComponentIndex={selectedUiNodeComponentIndex}
        selectedUiNodeEntityId={selectedUiNodeEntityId}
        selectedUiNodePath={selectedUiNodePath}
      />
    </div>
  );
}
