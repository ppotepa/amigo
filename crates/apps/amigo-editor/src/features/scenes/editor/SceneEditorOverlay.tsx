import type React from "react";
import type {
  SceneEditorEntity,
  SceneEditorMode,
  SceneEditorTool,
} from "./sceneEditorTypes";
import { SceneEditorGizmo } from "./SceneEditorGizmo";

export function SceneEditorOverlay({
  entities,
  mode,
  onBeginEntityDrag,
  onSelectEntity,
  selectedEntityId,
  tool,
}: {
  entities: SceneEditorEntity[];
  selectedEntityId: string | null;
  tool: SceneEditorTool;
  mode: SceneEditorMode;
  onSelectEntity: (entityId: string | null) => void;
  onBeginEntityDrag: (event: React.PointerEvent<HTMLButtonElement>, entity: SceneEditorEntity) => void;
}) {
  if (mode !== "edit") return null;

  const selectedEntity = entities.find((entity) => entity.id === selectedEntityId) ?? null;

  return (
    <div className="scene-editor-overlay">
      {entities.map((entity) => (
        <button
          key={entity.id}
          className={`scene-editor-entity-box ${entity.id === selectedEntityId ? "selected" : ""} ${entity.locked ? "locked" : ""}`}
          type="button"
          title={`${entity.name} · ${entity.componentTypes.join(", ")}`}
          style={{
            left: entity.bounds.x,
            top: entity.bounds.y,
            width: entity.bounds.width,
            height: entity.bounds.height,
          }}
          onClick={(event) => {
            event.stopPropagation();
            onSelectEntity(entity.id);
          }}
          onPointerDown={(event) => {
            if (tool !== "move" && tool !== "select") return;
            onBeginEntityDrag(event, entity);
          }}
        >
          <span className="scene-editor-entity-kind">{entity.kind}</span>
          <span className="scene-editor-entity-name">{entity.name}</span>
        </button>
      ))}
      {selectedEntity ? <SceneEditorGizmo entity={selectedEntity} /> : null}
    </div>
  );
}
