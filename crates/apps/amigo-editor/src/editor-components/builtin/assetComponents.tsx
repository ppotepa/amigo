import { AssetBrowserPanel } from "../../features/assets/AssetBrowserPanel";
import { defineEditorComponent } from "../componentDefinitionFactory";
import type { EditorComponentDefinition, EditorComponentLaunchContext } from "../componentTypes";
import { LEFT_DOCK, dockable } from "./shared";

export const AssetsBrowserComponent = defineEditorComponent<EditorComponentLaunchContext>()(
  dockable({
    id: "assets.browser",
    title: "Assets",
    debugSource: "src/features/assets/AssetBrowserPanel.tsx",
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
      ],
    },
    render: AssetBrowserPanel,
  }),
);

export const ASSET_COMPONENTS = [AssetsBrowserComponent] as const satisfies readonly EditorComponentDefinition<any>[];
