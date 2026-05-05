import type {
  SceneEditorEntity,
  SceneEditorPoint,
} from "./sceneEditorTypes";

export function hitTestSceneEntity(
  entities: SceneEditorEntity[],
  point: SceneEditorPoint,
): SceneEditorEntity | null {
  for (const entity of [...entities].reverse()) {
    if (!entity.visible || entity.locked) continue;
    if (pointInEntity(entity, point)) return entity;
  }
  return null;
}

export function pointInEntity(
  entity: SceneEditorEntity,
  point: SceneEditorPoint,
): boolean {
  return (
    point.x >= entity.bounds.x &&
    point.y >= entity.bounds.y &&
    point.x <= entity.bounds.x + entity.bounds.width &&
    point.y <= entity.bounds.y + entity.bounds.height
  );
}
