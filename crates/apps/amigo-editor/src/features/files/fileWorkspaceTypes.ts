import type { EditorProjectFileDto } from "../../api/dto";

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
  | "tile_ruleset"
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

export type FileWorkspaceRuleContext = {
  extension: string;
  fileName: string;
  normalizedPath: string;
};

export type FileWorkspaceRule = {
  id: WorkspaceFileKind;
  matches: (file: EditorProjectFileDto, context: FileWorkspaceRuleContext) => boolean;
  createDescriptor: (
    file: EditorProjectFileDto,
    context: FileWorkspaceRuleContext,
  ) => FileWorkspaceDescriptor;
};
