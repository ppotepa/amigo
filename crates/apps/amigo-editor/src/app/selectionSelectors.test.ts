import { describe, expect, it } from "vitest";
import { selectedFilePath, selectedSceneId } from "./selectionSelectors";

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
});
