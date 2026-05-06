import { describe, expect, it } from "vitest";
import { selectedEntityId, selectedFilePath, selectedSceneId, selectedUiNode } from "./selectionSelectors";

describe("selectionSelectors", () => {
  it("treats file-backed asset selection as a file path", () => {
    expect(selectedFilePath({
      kind: "asset",
      modId: "ink-wars",
      assetKey: "ink-wars/fonts/debug-placeholder",
      filePath: "fonts/debug-placeholder/font.yml",
    })).toBe("fonts/debug-placeholder/font.yml");
  });

  it("does not infer a scene from non-scene selections", () => {
    expect(selectedSceneId({
      kind: "asset",
      modId: "ink-wars",
      assetKey: "ink-wars/spritesheets/pen-blue-v1",
      filePath: "spritesheets/pen-blue-v1/spritesheet.yml",
    })).toBeNull();
  });

  it("returns scene and entity ids from ui node selection", () => {
    const selection = {
      kind: "uiNode" as const,
      modId: "they-are-rotten",
      sceneId: "main-menu",
      entityId: "main-menu-ui",
      componentIndex: 0,
      nodePath: "root.menu-card.start",
    };

    expect(selectedSceneId(selection)).toBe("main-menu");
    expect(selectedEntityId(selection)).toBe("main-menu-ui");
    expect(selectedUiNode(selection)?.nodePath).toBe("root.menu-card.start");
  });
});
