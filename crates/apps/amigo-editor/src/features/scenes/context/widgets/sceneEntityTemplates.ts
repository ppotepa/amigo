export type SceneEntityTemplateId =
  | "empty"
  | "sprite"
  | "tilemap"
  | "trigger"
  | "camera"
  | "spawnPoint";

export type SceneEntityTemplate = {
  id: SceneEntityTemplateId;
  label: string;
  description: string;
  defaultName: string;
  defaultYamlPreview: string;
  category: "basic" | "render2d" | "physics2d" | "camera" | "gameplay";
  componentTypes: string[];
  requiresAssetKind?: string[];
};

export const SCENE_ENTITY_TEMPLATES: SceneEntityTemplate[] = [
  {
    id: "empty",
    label: "Empty Entity",
    description: "Identity only; add components later.",
    defaultName: "entity",
    defaultYamlPreview: "id: entity\ncomponents: []",
    category: "basic",
    componentTypes: [],
  },
  {
    id: "sprite",
    label: "Sprite Entity",
    description: "Transform2D + Sprite2D.",
    defaultName: "sprite_entity",
    defaultYamlPreview: "id: sprite_entity\ncomponents:\n  - type: Transform2D\n  - type: Sprite2D",
    category: "render2d",
    componentTypes: ["Transform2D", "Sprite2D"],
    requiresAssetKind: ["image-2d", "sprite-sheet-2d"],
  },
  {
    id: "tilemap",
    label: "TileMap Entity",
    description: "Transform2D + TileMap2D.",
    defaultName: "tilemap_entity",
    defaultYamlPreview: "id: tilemap_entity\ncomponents:\n  - type: Transform2D\n  - type: TileMap2D",
    category: "render2d",
    componentTypes: ["Transform2D", "TileMap2D"],
    requiresAssetKind: ["tile-set-2d"],
  },
  {
    id: "trigger",
    label: "Trigger Entity",
    description: "Transform2D + Trigger2D.",
    defaultName: "trigger_zone",
    defaultYamlPreview: "id: trigger_zone\ncomponents:\n  - type: Transform2D\n  - type: Trigger2D",
    category: "physics2d",
    componentTypes: ["Transform2D", "Trigger2D"],
  },
  {
    id: "camera",
    label: "Camera Entity",
    description: "Transform2D + Camera2D.",
    defaultName: "camera_main",
    defaultYamlPreview: "id: camera_main\ncomponents:\n  - type: Transform2D\n  - type: Camera2D",
    category: "camera",
    componentTypes: ["Transform2D", "Camera2D"],
  },
  {
    id: "spawnPoint",
    label: "Spawn Point",
    description: "Transform2D + SpawnPoint marker.",
    defaultName: "spawn_point",
    defaultYamlPreview: "id: spawn_point\ncomponents:\n  - type: Transform2D\n  - type: SpawnPoint",
    category: "gameplay",
    componentTypes: ["Transform2D", "SpawnPoint"],
  },
];
