import type {
  EditorSceneEntityDto,
  EditorSceneHierarchyDto,
  EditorSceneObjectDto,
  EditorSceneSnapshotDto,
  ScenePreviewDto,
} from "../../../api/dto";
import type {
  SceneEditorEntity,
  SceneEditorEntityKind,
  SceneEditorModel,
  SceneEditorResolution,
  SceneEditorTransform,
} from "./sceneEditorTypes";

const DEFAULT_RESOLUTION: SceneEditorResolution = {
  width: 1920,
  height: 1080,
};

export function buildSceneEditorModel({
  hierarchy,
  preview,
  sceneId,
  snapshot,
}: {
  sceneId: string;
  preview?: ScenePreviewDto;
  hierarchy?: EditorSceneHierarchyDto;
  snapshot?: EditorSceneSnapshotDto | null;
}): SceneEditorModel {
  const resolution = previewResolution(preview);
  const entities = hierarchy?.entities ?? [];
  const entityById = new Map(entities.map((entity) => [entity.id, entity]));
  const snapshotEntities = snapshot?.objects
    .map((object) => {
      const source = entityById.get(object.entityId);
      return source ? editorEntityFromSnapshotObject(object, source) : null;
    })
    .filter((entity): entity is SceneEditorEntity => Boolean(entity)) ?? [];
  const layoutSource = snapshot
    ? snapshot.layoutSource === "runtime" || snapshot.layoutSource === "document"
      ? snapshot.layoutSource
      : "fallback"
    : "missingSnapshot";

  return {
    sceneId,
    resolution: snapshot ? { width: snapshot.width, height: snapshot.height } : resolution,
    entities: layoutSource === "runtime" || layoutSource === "document" ? snapshotEntities : [],
    layoutSource,
    quality: snapshot?.quality,
  };
}

export function applyDraftTransform(
  entity: SceneEditorEntity,
  draft?: Partial<SceneEditorTransform>,
): SceneEditorEntity {
  if (!draft) return entity;

  const transform = {
    ...entity.transform,
    ...draft,
  };

  return {
    ...entity,
    transform,
    bounds: {
      ...entity.bounds,
      x: transform.x - entity.bounds.width / 2,
      y: transform.y - entity.bounds.height / 2,
    },
    selectionBounds: entity.selectionBounds
      ? {
          ...entity.selectionBounds,
          x: transform.x - entity.selectionBounds.width / 2,
          y: transform.y - entity.selectionBounds.height / 2,
        }
      : entity.selectionBounds,
    renderBounds: entity.renderBounds
      ? {
          ...entity.renderBounds,
          x: transform.x - entity.renderBounds.width / 2,
          y: transform.y - entity.renderBounds.height / 2,
        }
      : entity.renderBounds,
  };
}

export function sceneEditorFrameUrl(preview?: ScenePreviewDto): string | null {
  return preview?.imageUrl ?? preview?.frameUrls[0] ?? null;
}

function previewResolution(preview?: ScenePreviewDto): SceneEditorResolution {
  if (preview?.width && preview?.height) {
    return {
      width: preview.width,
      height: preview.height,
    };
  }

  return DEFAULT_RESOLUTION;
}

function editorEntityFromSnapshotObject(
  object: EditorSceneObjectDto,
  source: EditorSceneEntityDto,
): SceneEditorEntity | null {
  const selectionBounds = object.selectionBounds2 ?? object.bounds2;
  if (!selectionBounds || !object.transform2) {
    return null;
  }

  const locked = object.locked || !object.selectable || !object.movable;

  return {
    id: object.entityId,
    name: object.name,
    source,
    kind: entityKind(source),
    visible: object.visible,
    locked,
    movable: object.movable,
    lockedReason: object.lockedReason,
    placementKind: object.placementKind,
    editCommandKind: object.editCommandKind,
    componentTypes: object.componentTypes,
    transform: object.transform2,
    bounds: selectionBounds,
    renderBounds: object.renderBounds2,
    selectionBounds,
  };
}

function entityKind(entity: EditorSceneEntityDto): SceneEditorEntityKind {
  const values = [
    entity.name,
    entity.id,
    ...entity.tags,
    ...entity.groups,
    ...entity.componentTypes,
  ].map((value) => value.toLowerCase());

  if (has(values, ["ui", "button", "panel", "layout"])) return "ui";
  if (has(values, ["camera"])) return "camera";
  if (has(values, ["script", "behavior", "state", "timer"])) return "script";
  if (has(values, ["physics", "collider", "trigger", "body"])) return "physics";
  if (has(values, ["motion", "velocity", "freeflight", "projectile"])) return "motion";
  if (has(values, ["sprite", "text", "vector", "material", "mesh", "render"])) return "render";
  if (has(values, ["audio", "sound", "music"])) return "audio";
  if (has(values, ["particle", "emitter"])) return "particles";
  if (has(values, ["tilemap", "tileset", "ruleset"])) return "tilemap";
  if (has(values, ["3d", "mesh3d", "text3d", "material3d"])) return "threed";
  return "other";
}

function has(values: string[], needles: string[]): boolean {
  return needles.some((needle) => values.some((value) => value.includes(needle)));
}
