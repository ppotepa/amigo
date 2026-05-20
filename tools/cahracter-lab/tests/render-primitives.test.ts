import test from "node:test";
import assert from "node:assert/strict";

import { sortRenderPrimitives, type RenderPrimitive } from "../src/render-primitives";

test("sortRenderPrimitives orders by pass then zIndex then depth", () => {
  const input: RenderPrimitive[] = [
    { id: "outline-b", sourcePartId: "b", kind: "path", role: "contour", layer: "outline", pass: "outline", zIndex: 10, depth: 0, visible: true },
    { id: "mass-near", sourcePartId: "near", kind: "path", role: "fill", layer: "rig", pass: "nearMass", zIndex: 5, depth: 20, visible: true },
    { id: "mass-far-a", sourcePartId: "far-a", kind: "path", role: "fill", layer: "rig", pass: "farMass", zIndex: 2, depth: 10, visible: true },
    { id: "mass-far-b", sourcePartId: "far-b", kind: "path", role: "fill", layer: "rig", pass: "farMass", zIndex: 2, depth: -5, visible: true },
    { id: "detail", sourcePartId: "detail", kind: "path", role: "detail", layer: "rig", pass: "detail", zIndex: 1, depth: 0, visible: true },
  ];

  const sorted = sortRenderPrimitives(input);

  assert.deepEqual(
    sorted.map(item => item.id),
    ["mass-far-b", "mass-far-a", "mass-near", "detail", "outline-b"],
  );
});
