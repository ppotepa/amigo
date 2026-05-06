import {
  FileCode2,
  FileImage,
  FolderPlus,
  Image as ImageIcon,
  LayoutPanelTop,
  Map as MapIcon,
  Menu,
  Monitor,
  Palette,
  Puzzle,
  Rows3,
  ScrollText,
  Type,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import type { AddItemKind, AddItemScope } from "./addItemTypes";

export interface AddItemDefinition {
  kind: AddItemKind;
  label: string;
  description: string;
  icon: LucideIcon;
  category: "project" | "assets" | "ui" | "scripts" | "advanced";
  defaultTargetPath: string;
  enabled: boolean;
  disabledReason?: string;
}

export const ADD_ITEM_DEFINITIONS: AddItemDefinition[] = [
  { kind: "scene", label: "Scene", description: "scene.yml + optional Rhai script", icon: MapIcon, category: "project", defaultTargetPath: "scenes", enabled: true },
  { kind: "ui-main-menu", label: "Main Menu UI", description: "Title menu template for Start / Options / Quit", icon: Menu, category: "ui", defaultTargetPath: "ui/menus", enabled: true },
  { kind: "ui-document", label: "UI Document", description: "Create a screen-space UiDocument template", icon: LayoutPanelTop, category: "ui", defaultTargetPath: "ui/documents", enabled: true },
  { kind: "ui-theme", label: "UI Theme", description: "Theme descriptor YAML", icon: Palette, category: "ui", defaultTargetPath: "ui/themes", enabled: true },
  { kind: "ui-hud", label: "HUD UI", description: "Coming soon", icon: Monitor, category: "ui", defaultTargetPath: "ui/hud", enabled: false, disabledReason: "Coming soon" },
  { kind: "ui-dialog", label: "Dialog UI", description: "Coming soon", icon: Rows3, category: "ui", defaultTargetPath: "ui/dialogs", enabled: false, disabledReason: "Coming soon" },
  { kind: "ui-component", label: "UI Component", description: "Coming soon", icon: Puzzle, category: "ui", defaultTargetPath: "ui/components", enabled: false, disabledReason: "Coming soon" },
  { kind: "script", label: "Script", description: "Rhai script file", icon: ScrollText, category: "scripts", defaultTargetPath: "scripts", enabled: true },
  { kind: "folder", label: "Folder", description: "Project folder", icon: FolderPlus, category: "advanced", defaultTargetPath: "", enabled: true },
  { kind: "raw-source", label: "Raw Source", description: "Copy source file into raw/", icon: FileImage, category: "assets", defaultTargetPath: "raw", enabled: true },
  { kind: "font", label: "Font", description: "fonts/<id>/font.yml", icon: Type, category: "assets", defaultTargetPath: "fonts", enabled: true },
  { kind: "image", label: "Image Descriptor", description: "Descriptor from raw image", icon: ImageIcon, category: "assets", defaultTargetPath: "spritesheets", enabled: true },
  { kind: "spritesheet", label: "Spritesheet", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "spritesheets", enabled: false, disabledReason: "Coming soon" },
  { kind: "tileset", label: "Tileset", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "spritesheets", enabled: false, disabledReason: "Coming soon" },
  { kind: "tilemap", label: "Tilemap", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "data/tilemaps", enabled: false, disabledReason: "Coming soon" },
  { kind: "audio", label: "Audio", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "audio", enabled: false, disabledReason: "Coming soon" },
  { kind: "prefab", label: "Prefab", description: "Coming soon", icon: FileCode2, category: "project", defaultTargetPath: "prefabs", enabled: false, disabledReason: "Coming soon" },
  { kind: "material", label: "Material", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "materials", enabled: false, disabledReason: "Coming soon" },
  { kind: "mesh", label: "Mesh", description: "Coming soon", icon: FileCode2, category: "assets", defaultTargetPath: "meshes", enabled: false, disabledReason: "Coming soon" },
];

export function defaultKindForScope(scope?: AddItemScope): AddItemKind | undefined {
  if (!scope) return undefined;
  if (scope.kind === "asset-category") {
    if (scope.category === "scenes") return "scene";
    if (scope.category === "ui") return "ui-main-menu";
    if (scope.category === "fonts") return "font";
    if (scope.category === "raw") return "raw-source";
    if (scope.category === "scripts") return "script";
    if (scope.category === "spritesheets") return "image";
  }
  return undefined;
}

export function catalogForScope(scope?: AddItemScope): AddItemDefinition[] {
  const all = ADD_ITEM_DEFINITIONS;
  if (!scope || scope.kind === "project-root") return all;
  if (scope.kind === "asset-category") {
    const map: Record<string, AddItemKind[]> = {
      scenes: ["scene"],
      ui: ["ui-main-menu", "ui-document", "ui-theme", "ui-hud", "ui-dialog", "ui-component"],
      fonts: ["font", "raw-source"],
      raw: ["raw-source"],
      spritesheets: ["image", "spritesheet", "tileset"],
      tilemaps: ["tilemap"],
      audio: ["audio", "raw-source"],
      scripts: ["script"],
    };
    const allowed = new Set(map[scope.category] ?? []);
    return all.filter((entry) => allowed.has(entry.kind));
  }
  return all;
}
