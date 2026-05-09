import { useMemo, useState } from "react";
import type { EditorSceneEntityDto } from "../../../api/dto";
import { ContextSearch } from "../../../ui/context-dock/ContextSearch";
import { ContextTree } from "../../../ui/context-dock/ContextTree";
import { WidgetFrame } from "../../../workbench/widgets/WidgetFrame";
import type { ContextTreeNode } from "../../../ui/context-dock/contextDockTypes";
import { sceneContextIcon } from "./sceneContextIcons";
import type { SceneEntityNode } from "./sceneContextTypes";

type EntityCategoryId =
  | "ui"
  | "camera"
  | "behavior"
  | "physics"
  | "motion"
  | "render"
  | "audio"
  | "particles"
  | "tilemap"
  | "input"
  | "threed"
  | "other";

type EntityCategoryDefinition = {
  id: EntityCategoryId;
  label: string;
  iconName: string;
  match: (entity: EditorSceneEntityDto) => boolean;
};

const ENTITY_CATEGORIES: EntityCategoryDefinition[] = [
  {
    id: "ui",
    label: "UI",
    iconName: "entity-ui",
    match: (entity) => hasAnyComponent(entity, ["ui", "button", "panel", "layout"]),
  },
  {
    id: "camera",
    label: "Cameras",
    iconName: "entity-camera",
    match: (entity) => hasAnyComponent(entity, ["camera"]),
  },
  {
    id: "behavior",
    label: "Behavior & Scripts",
    iconName: "entity-behavior",
    match: (entity) => hasAnyComponent(entity, ["script", "behavior", "state", "timer"]),
  },
  {
    id: "physics",
    label: "Physics",
    iconName: "entity-physics",
    match: (entity) => hasAnyComponent(entity, ["physics", "collider", "trigger", "body"]),
  },
  {
    id: "motion",
    label: "Motion",
    iconName: "entity-motion",
    match: (entity) => hasAnyComponent(entity, ["motion", "velocity", "freeflight", "projectile"]),
  },
  {
    id: "render",
    label: "Renderables",
    iconName: "entity-render",
    match: (entity) => hasAnyComponent(entity, ["sprite", "text", "vector", "material", "mesh", "render"]),
  },
  {
    id: "audio",
    label: "Audio",
    iconName: "audio",
    match: (entity) => hasAnyComponent(entity, ["audio", "sound", "music"]),
  },
  {
    id: "particles",
    label: "Particles",
    iconName: "entity-particles",
    match: (entity) => hasAnyComponent(entity, ["particle", "emitter"]),
  },
  {
    id: "tilemap",
    label: "Tilemaps",
    iconName: "tilemap",
    match: (entity) => hasAnyComponent(entity, ["tilemap", "tileset", "ruleset"]),
  },
  {
    id: "input",
    label: "Input",
    iconName: "entity-input",
    match: (entity) => hasAnyComponent(entity, ["input", "action"]),
  },
  {
    id: "threed",
    label: "3D",
    iconName: "entity-3d",
    match: (entity) => hasAnyComponent(entity, ["3d", "mesh3d", "text3d", "material3d"]),
  },
  {
    id: "other",
    label: "Other",
    iconName: "entity-other",
    match: () => true,
  },
];

export function SceneEntitiesWidget({
  entities,
  loading,
  onSelectEntity,
}: {
  entities: SceneEntityNode[];
  loading: boolean;
  onSelectEntity?: (entityId: string) => void;
}) {
  const [query, setQuery] = useState("");
  const treeNodes = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    const filtered = entities.filter(({ entity }) =>
      !normalized ||
      entity.name.toLowerCase().includes(normalized) ||
      entity.id.toLowerCase().includes(normalized) ||
      entity.componentTypes.some((component) => component.toLowerCase().includes(normalized)),
    );

    return entityNodesToTree(filtered, onSelectEntity);
  }, [entities, onSelectEntity, query]);

  return (
    <WidgetFrame
      id="scene-entities"
      title="Entities"
      icon={sceneContextIcon("entities")}
      badge={entities.length}
      badgeTone={entities.length ? "info" : "muted"}
      maxBodyHeight={360}
    >
      <ContextSearch value={query} placeholder="Search entities..." onChange={setQuery} />
      {loading ? (
        <p className="muted workspace-note">Indexing scene entities...</p>
      ) : (
        <ContextTree nodes={treeNodes} />
      )}
    </WidgetFrame>
  );
}

function entityNodesToTree(
  entities: SceneEntityNode[],
  onSelectEntity?: (entityId: string) => void,
): ContextTreeNode[] {
  if (!entities.length) return [];

  const categoryMap = new Map<EntityCategoryId, SceneEntityNode[]>();
  for (const node of entities) {
    const category = categoryForEntity(node.entity);
    const bucket = categoryMap.get(category.id) ?? [];
    bucket.push(node);
    categoryMap.set(category.id, bucket);
  }

  return ENTITY_CATEGORIES
    .map((category): ContextTreeNode | null => {
      const categoryEntities = categoryMap.get(category.id) ?? [];
      if (!categoryEntities.length) return null;

      return {
        id: `entities:${category.id}`,
        title: category.label,
        subtitle: `${categoryEntities.length} entities`,
        icon: sceneContextIcon(category.iconName),
        badge: categoryEntities.length,
        defaultExpanded: category.id === "ui" || category.id === "behavior" || category.id === "render",
        children: categoryEntities
          .sort((left, right) => left.entity.name.localeCompare(right.entity.name))
          .map(({ entity, selected }) => entityToTreeNode(entity, selected, onSelectEntity)),
      };
    })
    .filter((node): node is ContextTreeNode => Boolean(node));
}

function entityToTreeNode(
  entity: EditorSceneEntityDto,
  selected: boolean,
  onSelectEntity?: (entityId: string) => void,
): ContextTreeNode {
  const category = categoryForEntity(entity);
  return {
    id: entity.id,
    title: entity.name,
    subtitle: `${entity.componentCount} components${entity.tags.length ? ` · #${entity.tags.join(" #")}` : ""}`,
    icon: sceneContextIcon(category.iconName),
    badge: primaryComponentLabel(entity),
    selected,
    onSelect: () => onSelectEntity?.(entity.id),
  };
}

function categoryForEntity(entity: EditorSceneEntityDto): EntityCategoryDefinition {
  return ENTITY_CATEGORIES.find((category) => category.match(entity)) ?? ENTITY_CATEGORIES[ENTITY_CATEGORIES.length - 1];
}

function hasAnyComponent(entity: EditorSceneEntityDto, needles: string[]): boolean {
  const values = [
    entity.name,
    entity.id,
    ...entity.tags,
    ...entity.componentTypes,
  ].map((value) => value.toLowerCase());

  return needles.some((needle) => values.some((value) => value.includes(needle)));
}

function primaryComponentLabel(entity: EditorSceneEntityDto): string {
  return entity.componentTypes[0]?.replace(/_/g, " ") ?? "entity";
}
