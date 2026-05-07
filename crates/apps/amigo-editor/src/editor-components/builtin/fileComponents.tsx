import { FileWorkspaceHost } from "../../features/files/FileWorkspaceHost";
import { FilesBrowserPanel } from "../../features/files/FilesBrowserPanel";
import { ScriptsBrowserPanel } from "../../features/files/ScriptsBrowserPanel";
import type { EditorComponentDefinition, IconKey } from "../componentTypes";
import { BOTTOM_DOCK, CENTER_TAB, dockable, workspaceSurface } from "./shared";

const FILE_COMPONENTS_DATA: Array<{
  id: string;
  title: string;
  category: EditorComponentDefinition["category"];
  domain: EditorComponentDefinition["domain"];
  icon: IconKey;
}> = [
  { id: "file.manifest", title: "Manifest", category: "editor", domain: "modding", icon: "box" },
  { id: "file.scene", title: "Scene", category: "editor", domain: "scene", icon: "play" },
  { id: "file.scene-script", title: "Scene Script", category: "editor", domain: "scripting", icon: "terminal" },
  { id: "file.package", title: "Package", category: "editor", domain: "scripting", icon: "package" },
  { id: "file.script", title: "Script", category: "editor", domain: "scripting", icon: "terminal" },
  { id: "file.texture", title: "Texture", category: "preview", domain: "assets", icon: "image" },
  { id: "file.image-asset", title: "Image Asset", category: "editor", domain: "assets", icon: "image" },
  { id: "file.raw-image", title: "Raw Image", category: "preview", domain: "assets", icon: "image" },
  { id: "file.sprite", title: "Sprite", category: "editor", domain: "rendering_2d", icon: "image" },
  { id: "file.atlas", title: "Atlas", category: "editor", domain: "rendering_2d", icon: "image" },
  { id: "file.tileset", title: "Tileset", category: "editor", domain: "tileset", icon: "layers" },
  { id: "file.tile-ruleset", title: "Tile Ruleset", category: "editor", domain: "tileset", icon: "layers" },
  { id: "file.tilemap", title: "Tilemap", category: "editor", domain: "tilemap", icon: "layers" },
  { id: "file.config", title: "Config", category: "editor", domain: "editor", icon: "box" },
  { id: "file.text", title: "Text", category: "editor", domain: "editor", icon: "terminal" },
  { id: "file.binary", title: "Unsupported File", category: "workspace", domain: "assets", icon: "alert-triangle" },
];

export const FILE_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "files.browser",
    title: "Files",
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
  dockable({
    id: "scripts.browser",
    title: "Scripts",
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
  ...FILE_COMPONENTS_DATA.map((component) =>
    workspaceSurface({
      ...component,
      placement: CENTER_TAB,
      defaultPlacement: CENTER_TAB,
      allowedPlacements: ["centerTab", "floatingPanel", "window"],
      requiredContext: ["editorSession"],
      singleton: false,
      surface: {
        kind: component.category === "preview" ? "viewer" : "editor",
        tabMode: true,
        detachedMode: true,
        detachBehavior: "workspace",
        dockProfileId: component.category === "preview" ? "minimal" : "file-viewer",
      },
      render: FileWorkspaceHost,
    }),
  ),
];
