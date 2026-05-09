import { describe, expect, it } from "vitest";
import { SCENE_CONTEXT_WIDGETS } from "./widgets/sceneContextWidgets";

describe("scene context widgets", () => {
  it("registers widgets with unique ids", () => {
    const ids = SCENE_CONTEXT_WIDGETS.map((widget) => widget.id);

    expect(new Set(ids).size).toBe(ids.length);
  });

  it("keeps scene widgets inside the top context pane", () => {
    expect(SCENE_CONTEXT_WIDGETS.every((widget) => widget.placement === "top")).toBe(true);
  });

  it("registers only the intended top widgets", () => {
    expect(SCENE_CONTEXT_WIDGETS.map((widget) => widget.id)).toEqual([
      "scene.header",
      "scene.navigation",
      "scene.components",
      "scene.entities",
    ]);
  });

  it("defines status and folded hint selectors for compact rendering", () => {
    expect(SCENE_CONTEXT_WIDGETS.every((widget) => widget.getStatus)).toBe(true);
    expect(SCENE_CONTEXT_WIDGETS.every((widget) => widget.getFoldedHint)).toBe(true);
  });

  it("targets scene context only", () => {
    expect(SCENE_CONTEXT_WIDGETS.every((widget) => widget.targetKinds.includes("scene"))).toBe(true);
    expect(SCENE_CONTEXT_WIDGETS.every((widget) => Number.isFinite(widget.order))).toBe(true);
  });
});
