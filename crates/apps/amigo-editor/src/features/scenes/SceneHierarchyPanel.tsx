import type { EditorSceneHierarchyDto, EditorSceneSummaryDto } from "../../api/dto";
import type { EditorComponentProps } from "../../editor-components/componentTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export function SceneHierarchyPanel({
  services,
}: EditorComponentProps<WorkspaceRuntimeServices>) {
  return (
    <SceneHierarchy
      hierarchy={services.hierarchy}
      loading={services.hierarchyTask?.status === "running"}
      onSelectEntity={(entityId) => services.selectSceneEntity?.(entityId)}
      selectedEntityId={services.selectedEntity?.id ?? null}
      selectedScene={services.selectedScene ?? null}
    />
  );
}

function SceneHierarchy({
  selectedScene,
  hierarchy,
  loading,
  selectedEntityId,
  onSelectEntity,
}: {
  selectedScene: EditorSceneSummaryDto | null;
  hierarchy?: EditorSceneHierarchyDto;
  loading: boolean;
  selectedEntityId: string | null;
  onSelectEntity: (entityId: string) => void;
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
        hierarchy.entities.map((entity) => (
          <button
            key={entity.id}
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
        ))
      ) : (
        <p className="muted workspace-note">No authored entities found in this scene document.</p>
      )}
    </div>
  );
}

function SectionTitle({ title }: { title: string }) {
  return <div className="workspace-section-title">{title}</div>;
}

function Row({ icon, title, detail, badge, selected }: { icon: string; title: string; detail: string; badge?: string; selected?: boolean }) {
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
