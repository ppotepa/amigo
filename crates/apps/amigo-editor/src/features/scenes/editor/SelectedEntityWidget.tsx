import type {
  EditorSceneEntityDto,
  EditorSceneObjectDto,
} from "../../../api/dto";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import { WidgetRow } from "../../../workbench/widgets/WidgetRow";
import { sceneIcon } from "../../scene/sceneIcons";

export function SelectedEntityWidget({
  entity,
  object,
}: {
  entity: EditorSceneEntityDto | null;
  object: EditorSceneObjectDto | null;
}) {
  if (!entity) {
    return (
      <WidgetFrame
        id="selected-entity"
        title="Selected Entity"
        icon={sceneIcon("entity")}
        badge="none"
        badgeTone="muted"
        defaultCollapsed
      >
        <p className="muted workspace-note">No entity selected.</p>
      </WidgetFrame>
    );
  }

  return (
    <WidgetFrame
      id="selected-entity"
      title="Selected Entity"
      icon={sceneIcon("entity")}
      badge={entity.visible ? "visible" : "hidden"}
      badgeTone={entity.visible ? "valid" : "muted"}
    >
      <WidgetRow
        icon={sceneIcon(object?.category ?? "entity")}
        title={entity.name}
        subtitle={entity.id}
        badge={entity.componentTypes[0] ?? "entity"}
        badgeTone="info"
      />
      <WidgetRow
        icon={sceneIcon("component")}
        title="Components"
        subtitle={entity.componentTypes.join(", ") || "No components"}
        badge={entity.componentCount}
      />
      <WidgetRow
        icon={sceneIcon("entity")}
        title="Tags"
        subtitle={entity.tags.length ? entity.tags.join(", ") : "No tags"}
        badge={entity.tags.length}
      />
      {object?.prefabInstance ? (
        <WidgetRow
          icon={sceneIcon("asset")}
          title="Prefab"
          subtitle={object.prefabInstance.prefabId}
          badge={object.prefabInstance.isPrefabRoot ? "root" : "child"}
          badgeTone="info"
        />
      ) : null}
    </WidgetFrame>
  );
}
