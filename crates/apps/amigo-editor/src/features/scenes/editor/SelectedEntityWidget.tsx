import type {
  EditorSceneEntityDto,
  EditorSceneObjectDto,
} from "../../../api/dto";
import { ContextRow } from "../../../ui/context-dock/ContextRow";
import { ContextWidget } from "../../../ui/context-dock/ContextWidget";
import { sceneContextIcon } from "../context/sceneContextIcons";

export function SelectedEntityWidget({
  entity,
  object,
}: {
  entity: EditorSceneEntityDto | null;
  object: EditorSceneObjectDto | null;
}) {
  if (!entity) {
    return (
      <ContextWidget
        id="selected-entity"
        title="Selected Entity"
        icon={sceneContextIcon("entity")}
        badge="none"
        badgeTone="muted"
        defaultCollapsed
      >
        <p className="muted workspace-note">No entity selected.</p>
      </ContextWidget>
    );
  }

  return (
    <ContextWidget
      id="selected-entity"
      title="Selected Entity"
      icon={sceneContextIcon("entity")}
      badge={entity.visible ? "visible" : "hidden"}
      badgeTone={entity.visible ? "valid" : "muted"}
    >
      <ContextRow
        icon={sceneContextIcon(object?.category ?? "entity")}
        title={entity.name}
        subtitle={entity.id}
        badge={entity.componentTypes[0] ?? "entity"}
        tone="cyan"
      />
      <ContextRow
        icon={sceneContextIcon("component")}
        title="Components"
        subtitle={entity.componentTypes.join(", ") || "No components"}
        badge={entity.componentCount}
      />
      <ContextRow
        icon={sceneContextIcon("entity")}
        title="Tags"
        subtitle={entity.tags.length ? entity.tags.join(", ") : "No tags"}
        badge={entity.tags.length}
      />
    </ContextWidget>
  );
}
