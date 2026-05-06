import { AssetBrowserPanel } from "../../features/assets/AssetBrowserPanel";
import type { EditorComponentDefinition } from "../componentTypes";
import { LEFT_DOCK, dockable } from "./shared";

export const ASSET_COMPONENTS: EditorComponentDefinition[] = [
  dockable({
    id: "assets.browser",
    title: "Assets",
    category: "explorer",
    domain: "assets",
    icon: "package",
    description: "Asset type browser and future thumbnail index.",
    placement: LEFT_DOCK,
    defaultPlacement: LEFT_DOCK,
    allowedPlacements: ["leftDock", "rightDock", "floatingPanel"],
    requiredContext: ["editorSession"],
    toolbar: {
      compact: true,
      controls: [
        { kind: "action", id: "add", label: "Add Item", icon: "plus" },
        { kind: "spacer", id: "spacer-main" },
        {
          kind: "select",
          id: "kind",
          label: "Filter",
          defaultValue: "all",
          options: [
            { id: "all", label: "All" },
            { id: "image-2d", label: "Images" },
            { id: "tileset-2d", label: "Tilesets" },
            { id: "tile-ruleset-2d", label: "Rulesets" },
            { id: "tilemap-2d", label: "Tilemaps" },
            { id: "sprite-sheet-2d", label: "Sprites" },
          ],
        },
        {
          kind: "segmented",
          id: "viewMode",
          label: "View",
          defaultValue: "tree",
          options: [
            { id: "tree", label: "Tree", icon: "list-tree" },
            { id: "list", label: "List", icon: "list" },
            { id: "tiles", label: "Tiles", icon: "grid" },
          ],
        },
      ],
    },
    render: AssetBrowserPanel,
  }),
];
