import { FileWorkspaceHost } from "../../features/files/FileWorkspaceHost";
import { FilesBrowserPanel } from "../../features/files/FilesBrowserPanel";
import { ScriptsBrowserPanel } from "../../features/files/ScriptsBrowserPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type {
  EditorComponentDefinition,
  EditorComponentLaunchContext,
  FileWorkspaceComponentContext,
} from "../componentTypes";
import { BOTTOM_DOCK, CENTER_TAB, dockable, workspaceSurface } from "./shared";

export const FilesBrowserComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "files.browser",
    title: "Files",
    debugSource: "src/features/files/FilesBrowserPanel.tsx",
    category: "explorer",
    domain: "project",
    icon: "folder",
    description: "Raw project filesystem view.",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    toolbar: {
      compact: true,
      controls: [
        {
          kind: "segmented",
          id: "viewMode",
          label: "View",
          defaultValue: "tree",
          options: [
            { id: "tree", label: "Tree", icon: "list-tree" },
            { id: "flat", label: "Flat", icon: "list" },
          ],
        },
        {
          kind: "select",
          id: "fileFilter",
          label: "Filter",
          defaultValue: "all",
          options: [
            { id: "all", label: "All" },
            { id: "descriptors", label: "Descriptors" },
            { id: "raw", label: "Raw" },
            { id: "scripts", label: "Scripts" },
            { id: "scenes", label: "Scenes" },
          ],
        },
      ],
    },
    render: FilesBrowserPanel,
  }),
);

export const ScriptsBrowserComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "scripts.browser",
    title: "Scripts",
    debugSource: "src/features/files/ScriptsBrowserPanel.tsx",
    category: "explorer",
    domain: "scripting",
    icon: "terminal",
    description: "Scripts and packages navigation.",
    placement: BOTTOM_DOCK,
    defaultPlacement: BOTTOM_DOCK,
    allowedPlacements: ["bottomDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    toolbar: {
      compact: true,
      controls: [
        {
          kind: "segmented",
          id: "viewMode",
          label: "View",
          defaultValue: "tree",
          options: [
            { id: "tree", label: "Tree", icon: "list-tree" },
            { id: "packages", label: "Packages", icon: "package" },
            { id: "flat", label: "Flat", icon: "list" },
          ],
        },
        { kind: "toggle", id: "packagesOnly", label: "Packages", icon: "package", defaultValue: false },
      ],
    },
    render: ScriptsBrowserPanel,
  }),
);

function fileWorkspaceComponent(input: {
  id: string;
  title: string;
  debugSource?: string;
  category: EditorComponentDefinition["category"];
  domain: EditorComponentDefinition["domain"];
  icon: EditorComponentDefinition["icon"];
}) {
  return defineEditorComponent<FileWorkspaceComponentContext>()(
    workspaceSurface({
      ...input,
      debugSource: input.debugSource ?? "src/features/files/FileWorkspaceHost.tsx",
      placement: CENTER_TAB,
      defaultPlacement: CENTER_TAB,
      allowedPlacements: ["centerTab", "floatingPanel"],
      requiredContext: ["editorSession"],
      singleton: false,
      surface: {
        kind: input.category === "preview" ? "viewer" : "editor",
        tabMode: true,
        detachedMode: true,
        detachBehavior: "workspace",
        dockProfileId: input.category === "preview" ? "minimal" : "file-viewer",
      },
      render: FileWorkspaceHost,
    }),
  );
}

export const FileManifestComponent = fileWorkspaceComponent({ id: "file.manifest", title: "Manifest", category: "editor", domain: "modding", icon: "box" });
export const FileSceneComponent = fileWorkspaceComponent({ id: "file.scene", title: "Scene", category: "editor", domain: "scene", icon: "play" });
export const FileSceneScriptComponent = fileWorkspaceComponent({ id: "file.scene-script", title: "Scene Script", category: "editor", domain: "scripting", icon: "terminal" });
export const FilePackageComponent = fileWorkspaceComponent({ id: "file.package", title: "Package", category: "editor", domain: "scripting", icon: "package" });
export const FileScriptComponent = fileWorkspaceComponent({ id: "file.script", title: "Script", category: "editor", domain: "scripting", icon: "terminal" });
export const FileTextureComponent = fileWorkspaceComponent({ id: "file.texture", title: "Texture", category: "preview", domain: "assets", icon: "image" });
export const FileImageAssetComponent = fileWorkspaceComponent({ id: "file.image-asset", title: "Image Asset", category: "editor", domain: "assets", icon: "image" });
export const FileRawImageComponent = fileWorkspaceComponent({ id: "file.raw-image", title: "Raw Image", category: "preview", domain: "assets", icon: "image" });
export const FileSpriteComponent = fileWorkspaceComponent({ id: "file.sprite", title: "Sprite", category: "editor", domain: "rendering_2d", icon: "image" });
export const FileAtlasComponent = fileWorkspaceComponent({ id: "file.atlas", title: "Atlas", category: "editor", domain: "rendering_2d", icon: "image" });
export const FileTilesetComponent = fileWorkspaceComponent({ id: "file.tileset", title: "Tileset", category: "editor", domain: "tileset", icon: "layers" });
export const FileTileRulesetComponent = fileWorkspaceComponent({ id: "file.tile-ruleset", title: "Tile Ruleset", category: "editor", domain: "tileset", icon: "layers" });
export const FileTilemapComponent = fileWorkspaceComponent({ id: "file.tilemap", title: "Tilemap", category: "editor", domain: "tilemap", icon: "layers" });
export const FileConfigComponent = fileWorkspaceComponent({ id: "file.config", title: "Config", category: "editor", domain: "editor", icon: "box" });
export const FileTextComponent = fileWorkspaceComponent({ id: "file.text", title: "Text", category: "editor", domain: "editor", icon: "terminal" });
export const FileBinaryComponent = fileWorkspaceComponent({ id: "file.binary", title: "Unsupported File", category: "workspace", domain: "assets", icon: "alert-triangle" });

export const FILE_COMPONENTS = [
  FilesBrowserComponent,
  ScriptsBrowserComponent,
  FileManifestComponent,
  FileSceneComponent,
  FileSceneScriptComponent,
  FilePackageComponent,
  FileScriptComponent,
  FileTextureComponent,
  FileImageAssetComponent,
  FileRawImageComponent,
  FileSpriteComponent,
  FileAtlasComponent,
  FileTilesetComponent,
  FileTileRulesetComponent,
  FileTilemapComponent,
  FileConfigComponent,
  FileTextComponent,
  FileBinaryComponent,
] as const satisfies readonly EditorComponentDefinition<any>[];
