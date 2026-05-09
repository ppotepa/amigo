import { describe, expect, it } from "vitest";
import { SCENE_ENTITY_TEMPLATES } from "./sceneEntityTemplates";

describe("scene entity templates", () => {
  it("uses unique ids", () => {
    const ids = SCENE_ENTITY_TEMPLATES.map((template) => template.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("defines expected component bundles", () => {
    const sprite = SCENE_ENTITY_TEMPLATES.find((template) => template.id === "sprite");
    const trigger = SCENE_ENTITY_TEMPLATES.find((template) => template.id === "trigger");

    expect(sprite?.componentTypes).toEqual(expect.arrayContaining(["Transform2D", "Sprite2D"]));
    expect(trigger?.componentTypes).toEqual(expect.arrayContaining(["Transform2D", "Trigger2D"]));
  });

  it("exposes asset requirements for asset-backed templates", () => {
    const sprite = SCENE_ENTITY_TEMPLATES.find((template) => template.id === "sprite");
    expect(sprite?.requiresAssetKind).toContain("image-2d");
  });
});
