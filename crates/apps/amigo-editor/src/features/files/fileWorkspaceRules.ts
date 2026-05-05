import type { EditorProjectFileContentDto, EditorProjectFileDto } from "../../api/dto";
import { TEXT_EXTENSIONS } from "./fileContentRules";
import { fileExtension, normalizePath } from "./filePathUtils";
import type {
  FileWorkspaceDescriptor,
  FileWorkspaceRule,
  WorkspaceFileKind,
  WorkspaceOpenMode,
  WorkspaceShape,
} from "./fileWorkspaceTypes";

const IMAGE_EXTENSIONS = new Set([".png", ".jpg", ".jpeg", ".webp"]);

export const FILE_WORKSPACE_RULES: readonly FileWorkspaceRule[] = [
  {
    id: "manifest",
    matches: (_file, { fileName }) => fileName === "mod.toml",
    createDescriptor: () =>
      descriptor("manifest", "file.manifest", "form-plus-source", "editorViewer", "Manifest", "T", true),
  },
  {
    id: "script_package",
    matches: (_file, { fileName }) => fileName === "package.yml" || fileName === "package.yaml",
    createDescriptor: () =>
      descriptor("script_package", "file.package", "form-plus-source", "editorViewer", "Package", "Pkg", true),
  },
  {
    id: "scene_document",
    matches: (_file, { fileName, normalizedPath }) =>
      fileName === "scene.yml" ||
      fileName === "scene.yaml" ||
      normalizedPath.endsWith(".scene.yml") ||
      normalizedPath.endsWith(".scene.yaml"),
    createDescriptor: () =>
      descriptor("scene_document", "file.scene", "form-plus-source", "editorViewer", "Scene", "Sc", true),
  },
  {
    id: "scene_script",
    matches: (_file, { fileName, normalizedPath }) =>
      fileName === "scene.rhai" || normalizedPath.endsWith(".scene.rhai"),
    createDescriptor: () =>
      descriptor("scene_script", "file.scene-script", "text-editor", "editor", "Scene Script", "Rh", true),
  },
  {
    id: "tile_ruleset",
    matches: (_file, { normalizedPath }) =>
      normalizedPath.endsWith(".tile-ruleset.yml") ||
      normalizedPath.endsWith(".tile-ruleset.yaml") ||
      isSpritesheetSubasset(normalizedPath, "rulesets"),
    createDescriptor: () =>
      descriptor("tile_ruleset", "file.tile-ruleset", "canvas-editor", "editorViewer", "Tile Ruleset", "Rule", true),
  },
  {
    id: "tileset",
    matches: (file, { normalizedPath }) =>
      normalizedPath.endsWith(".tileset.yml") ||
      normalizedPath.endsWith(".tileset.yaml") ||
      isSpritesheetSubasset(normalizedPath, "tilesets") ||
      file.kind === "tileset",
    createDescriptor: () =>
      descriptor("tileset", "file.tileset", "canvas-editor", "editorViewer", "Tileset", "Ts", true),
  },
  {
    id: "tilemap",
    matches: (file, { normalizedPath }) =>
      normalizedPath.endsWith(".tilemap.yml") ||
      normalizedPath.endsWith(".tilemap.yaml") ||
      file.kind === "tilemap",
    createDescriptor: () =>
      descriptor("tilemap", "file.tilemap", "canvas-editor", "editorViewer", "Tilemap", "Tm", true),
  },
  {
    id: "spritesheet",
    matches: (_file, { normalizedPath }) =>
      normalizedPath.endsWith(".sprite.yml") ||
      normalizedPath.endsWith(".sprite.yaml") ||
      (normalizedPath.endsWith("spritesheet.yml") && normalizedPath.startsWith("spritesheets/")),
    createDescriptor: () =>
      descriptor("spritesheet", "file.sprite", "preview-plus-inspector", "editorViewer", "Sprite", "Sp", true),
  },
  {
    id: "atlas",
    matches: (_file, { normalizedPath }) =>
      normalizedPath.endsWith(".atlas.yml") || normalizedPath.endsWith(".atlas.yaml"),
    createDescriptor: () =>
      descriptor("atlas", "file.atlas", "preview-plus-inspector", "editorViewer", "Atlas", "At", true),
  },
  {
    id: "image_asset",
    matches: (file, { normalizedPath }) =>
      normalizedPath.endsWith(".image.yml") ||
      normalizedPath.endsWith(".image.yaml") ||
      file.kind === "imageAsset",
    createDescriptor: () =>
      descriptor("image_asset", "file.image-asset", "form-plus-source", "editorViewer", "Image Asset", "Img", true),
  },
  {
    id: "script",
    matches: (file) => file.kind === "script",
    createDescriptor: () =>
      descriptor("script", "file.script", "text-editor", "editor", "Script", "Rh", true),
  },
  {
    id: "spritesheet",
    matches: (file) => file.kind === "spritesheet",
    createDescriptor: () =>
      descriptor("spritesheet", "file.sprite", "preview-plus-inspector", "viewer", "Spritesheet", "Sp", false),
  },
  {
    id: "raw_image",
    matches: (file, { extension }) =>
      file.kind === "rawImage" || file.kind === "texture" || IMAGE_EXTENSIONS.has(extension),
    createDescriptor: () =>
      descriptor("raw_image", "file.raw-image", "preview-plus-inspector", "viewer", "Raw Image", "Img", false),
  },
  {
    id: "config",
    matches: (_file, { extension }) =>
      extension === ".toml" || extension === ".yml" || extension === ".yaml",
    createDescriptor: () =>
      descriptor("config", "file.config", "text-editor", "editor", "Config", "Cfg", true),
  },
  {
    id: "unknown_text",
    matches: (_file, { extension }) => TEXT_EXTENSIONS.has(extension),
    createDescriptor: () =>
      descriptor("unknown_text", "file.text", "text-editor", "editor", "Text", "Txt", true),
  },
];

export function resolveFileWorkspaceDescriptor(file: EditorProjectFileDto): FileWorkspaceDescriptor {
  const context = {
    extension: fileExtension(file.name),
    fileName: file.name.toLowerCase(),
    normalizedPath: normalizePath(file.relativePath).toLowerCase(),
  };

  return (
    FILE_WORKSPACE_RULES.find((rule) => rule.matches(file, context))?.createDescriptor(file, context) ??
    descriptor("unknown_binary", "file.binary", "unsupported", "unsupported", "Unsupported", "Bin", false)
  );
}

export function workspaceDescriptorLanguage(
  descriptor: FileWorkspaceDescriptor,
  content?: Pick<EditorProjectFileContentDto, "language">,
): string {
  if (content?.language) {
    return content.language;
  }
  switch (descriptor.fileKind) {
    case "manifest":
      return "toml";
    case "scene_document":
    case "script_package":
    case "image_asset":
    case "tilemap":
    case "tileset":
    case "tile_ruleset":
    case "atlas":
    case "config":
      return "yaml";
    case "script":
    case "scene_script":
      return "rhai";
    default:
      return "text";
  }
}

function isSpritesheetSubasset(normalizedPath: string, subfolder: "tilesets" | "rulesets"): boolean {
  return (
    normalizedPath.startsWith("spritesheets/") &&
    normalizedPath.includes(`/${subfolder}/`) &&
    (normalizedPath.endsWith(".yml") || normalizedPath.endsWith(".yaml"))
  );
}

function descriptor(
  fileKind: WorkspaceFileKind,
  componentId: string,
  shape: WorkspaceShape,
  openMode: WorkspaceOpenMode,
  title: string,
  iconText: string,
  editable: boolean,
): FileWorkspaceDescriptor {
  return { fileKind, componentId, shape, openMode, title, iconText, editable };
}
