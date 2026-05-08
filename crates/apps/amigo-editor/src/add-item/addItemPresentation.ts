import type { AddItemKind } from "./addItemTypes";

export type AddItemCategory = "project" | "assets" | "ui" | "scripts" | "advanced";

export const ADD_ITEM_CATEGORY_LABELS = {
  project: "Project",
  assets: "Assets",
  ui: "UI",
  scripts: "Scripts",
  advanced: "Advanced",
} satisfies Record<AddItemCategory, string>;

export function toneForAddItemKind(kind: AddItemKind): string {
  switch (kind) {
    case "scene":
      return "asset-scene";
    case "ui-theme":
    case "ui-document":
    case "ui-main-menu":
    case "ui-hud":
    case "ui-dialog":
    case "ui-component":
      return "domain-theme";
    case "script":
      return "asset-script";
    case "folder":
      return "domain-project";
    case "raw-source":
      return "asset-raw-image";
    case "font":
      return "asset-font";
    case "image":
      return "asset-image";
    case "spritesheet":
      return "asset-sprite";
    case "tileset":
      return "asset-tileset";
    case "tilemap":
      return "asset-tilemap";
    case "audio":
      return "asset-audio";
    case "material":
    case "mesh":
      return "domain-rendering_2d";
    case "prefab":
    default:
      return "asset-generic";
  }
}

export function detailForAddItemKind(kind: AddItemKind): {
  creates: string[];
  useFor: string[];
  notes: string[];
} {
  switch (kind) {
    case "scene":
      return {
        creates: ["scene.yml document", "Optional scene.rhai script", "mod.toml scene entry"],
        useFor: ["Gameplay levels", "Runtime menus", "Previewable scene states"],
        notes: ["Best default choice when adding a new playable or previewable area."],
      };
    case "ui-theme":
      return {
        creates: ["UI theme YAML descriptor"],
        useFor: ["Runtime UI palettes", "Menu visual themes", "Reusable UI color sets"],
        notes: ["Theme editing is descriptor-first until the visual UI editor lands."],
      };
    case "ui-document":
      return {
        creates: ["Reusable UiDocument YAML"],
        useFor: ["Screen-space UI documents", "HUD prototypes", "Reusable runtime UI layouts"],
        notes: ["This creates a template asset, not a scene attachment."],
      };
    case "ui-main-menu":
      return {
        creates: ["Main menu UI YAML template"],
        useFor: ["Start / Options / Quit screens", "Runtime menu prototypes", "Menu layout reuse"],
        notes: ["The template is data-driven and can be referenced by a scene later."],
      };
    case "ui-hud":
    case "ui-dialog":
    case "ui-component":
      return {
        creates: ["UI template descriptor"],
        useFor: ["Future visual UI editor workflows", "Reusable runtime UI pieces"],
        notes: ["Coming soon."],
      };
    case "script":
      return {
        creates: ["Rhai script file"],
        useFor: ["Scene logic", "Runtime event hooks", "Prototype behavior"],
        notes: ["Scene-owned scripts should usually live beside their scene."],
      };
    case "folder":
      return {
        creates: ["Project folder"],
        useFor: ["Organizing source assets", "Grouping descriptors", "Preparing future asset domains"],
        notes: ["Folders are created only inside the active mod project."],
      };
    case "raw-source":
      return {
        creates: ["Copied source file in raw/"],
        useFor: ["Imported art/audio/source material", "Files that will later become typed assets"],
        notes: ["Raw files are not runtime descriptors until a typed asset references them."],
      };
    case "font":
      return {
        creates: ["fonts/<id>/font.yml"],
        useFor: ["Text2D", "Runtime UI text", "Menu typography"],
        notes: ["The first version creates a placeholder descriptor."],
      };
    case "image":
      return {
        creates: ["Image, sprite, or tileset descriptor from a raw image"],
        useFor: ["2D backgrounds", "Spritesheets", "Tileset sources"],
        notes: ["Requires selecting an existing raw image source."],
      };
    case "spritesheet":
      return {
        creates: ["Spritesheet descriptor"],
        useFor: ["Frame grids", "Animated 2D sprites", "Sprite libraries"],
        notes: ["Coming soon."],
      };
    case "tileset":
      return {
        creates: ["Tileset descriptor"],
        useFor: ["Tile palettes", "Tilemap painting", "Grid based maps"],
        notes: ["Coming soon."],
      };
    case "tilemap":
      return {
        creates: ["Tilemap data document"],
        useFor: ["Level collision/layout maps", "Tile based scenes", "2D tactical maps"],
        notes: ["Coming soon."],
      };
    case "audio":
      return {
        creates: ["Audio descriptor"],
        useFor: ["Music", "SFX", "Runtime audio references"],
        notes: ["Coming soon."],
      };
    case "prefab":
      return {
        creates: ["Prefab document"],
        useFor: ["Reusable entity trees", "UI widgets", "Spawnable game objects"],
        notes: ["Coming soon."],
      };
    case "material":
      return {
        creates: ["Material descriptor"],
        useFor: ["3D rendering", "Shader/material references", "Reusable visual surfaces"],
        notes: ["Coming soon."],
      };
    case "mesh":
      return {
        creates: ["Mesh descriptor"],
        useFor: ["3D models", "Imported geometry", "Reusable mesh assets"],
        notes: ["Coming soon."],
      };
    default:
      return {
        creates: ["Project item"],
        useFor: ["Project content"],
        notes: [],
      };
  }
}
