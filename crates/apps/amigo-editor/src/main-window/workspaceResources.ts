import type { EditorProjectFileContentDto, EditorProjectFileDto } from "../api/dto";

export type WorkspaceOpenMode = "viewer" | "editor" | "editorViewer" | "unsupported";

export type WorkspaceShape =
  | "text-editor"
  | "form-plus-source"
  | "preview-plus-inspector"
  | "canvas-editor"
  | "runtime-preview"
  | "unsupported";

export type WorkspaceFileKind =
  | "manifest"
  | "scene_document"
  | "scene_script"
  | "script"
  | "script_package"
  | "image_asset"
  | "raw_image"
  | "texture"
  | "spritesheet"
  | "tilemap"
  | "tileset"
  | "atlas"
  | "config"
  | "unknown_text"
  | "unknown_binary";

export type FileWorkspaceDescriptor = {
  fileKind: WorkspaceFileKind;
  componentId: string;
  openMode: WorkspaceOpenMode;
  shape: WorkspaceShape;
  title: string;
  iconText: string;
  editable: boolean;
};

const TEXT_EXTENSIONS = new Set([".toml", ".yml", ".yaml", ".rhai", ".json", ".md", ".txt", ".ron"]);
const IMAGE_EXTENSIONS = new Set([".png", ".jpg", ".jpeg", ".webp"]);

export function resolveFileWorkspaceDescriptor(file: EditorProjectFileDto): FileWorkspaceDescriptor {
  const normalizedPath = normalizePath(file.relativePath).toLowerCase();
  const fileName = file.name.toLowerCase();
  const extension = fileExtension(file.name);

  if (fileName === "mod.toml") {
    return descriptor("manifest", "file.manifest", "form-plus-source", "editorViewer", "Manifest", "T", true);
  }

  if (fileName === "package.yml" || fileName === "package.yaml") {
    return descriptor("script_package", "file.package", "form-plus-source", "editorViewer", "Package", "Pkg", true);
  }

  if (fileName === "scene.yml" || fileName === "scene.yaml" || normalizedPath.endsWith(".scene.yml") || normalizedPath.endsWith(".scene.yaml")) {
    return descriptor("scene_document", "file.scene", "form-plus-source", "editorViewer", "Scene", "Sc", true);
  }

  if (fileName === "scene.rhai" || normalizedPath.endsWith(".scene.rhai")) {
    return descriptor("scene_script", "file.scene-script", "text-editor", "editor", "Scene Script", "Rh", true);
  }

  if (
    normalizedPath.endsWith(".tileset.yml") ||
    normalizedPath.endsWith(".tileset.yaml") ||
    normalizedPath.endsWith(".tile-ruleset.yml") ||
    normalizedPath.endsWith(".tile-ruleset.yaml") ||
    file.kind === "tileset"
  ) {
    return descriptor("tileset", "file.tileset", "canvas-editor", "editorViewer", "Tileset", "Ts", true);
  }

  if (normalizedPath.endsWith(".tilemap.yml") || normalizedPath.endsWith(".tilemap.yaml") || file.kind === "tilemap") {
    return descriptor("tilemap", "file.tilemap", "canvas-editor", "editorViewer", "Tilemap", "Tm", true);
  }

  if (normalizedPath.endsWith(".sprite.yml") || normalizedPath.endsWith(".sprite.yaml")) {
    return descriptor("spritesheet", "file.sprite", "preview-plus-inspector", "editorViewer", "Sprite", "Sp", true);
  }

  if (normalizedPath.endsWith(".atlas.yml") || normalizedPath.endsWith(".atlas.yaml")) {
    return descriptor("atlas", "file.atlas", "preview-plus-inspector", "editorViewer", "Atlas", "At", true);
  }

  if (normalizedPath.endsWith(".image.yml") || normalizedPath.endsWith(".image.yaml") || file.kind === "imageAsset") {
    return descriptor("image_asset", "file.image-asset", "form-plus-source", "editorViewer", "Image Asset", "Img", true);
  }

  if (file.kind === "script") {
    return descriptor("script", "file.script", "text-editor", "editor", "Script", "Rh", true);
  }

  if (file.kind === "spritesheet") {
    return descriptor("spritesheet", "file.sprite", "preview-plus-inspector", "viewer", "Spritesheet", "Sp", false);
  }

  if (file.kind === "rawImage" || file.kind === "texture" || IMAGE_EXTENSIONS.has(extension)) {
    return descriptor("raw_image", "file.raw-image", "preview-plus-inspector", "viewer", "Raw Image", "Img", false);
  }

  if (extension === ".toml" || extension === ".yml" || extension === ".yaml") {
    return descriptor("config", "file.config", "text-editor", "editor", "Config", "Cfg", true);
  }

  if (TEXT_EXTENSIONS.has(extension)) {
    return descriptor("unknown_text", "file.text", "text-editor", "editor", "Text", "Txt", true);
  }

  return descriptor("unknown_binary", "file.binary", "unsupported", "unsupported", "Unsupported", "Bin", false);
}

export function canReadProjectFileContent(file: EditorProjectFileDto): boolean {
  const extension = fileExtension(file.name);
  return TEXT_EXTENSIONS.has(extension);
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

function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

function fileExtension(fileName: string): string {
  const index = fileName.lastIndexOf(".");
  return index >= 0 ? fileName.slice(index).toLowerCase() : "";
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
