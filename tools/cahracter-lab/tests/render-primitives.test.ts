import test from "node:test";
import assert from "node:assert/strict";

import { computeRenderPassPolicy } from "../src/policies/layer";
import { computeRenderPasses } from "../src/scene-graph";
import type { EvaluatedBodyPart } from "../src/types";
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

test("computeRenderPassPolicy depth-sorts masses by camera depth", () => {
  const mass = { id: "ear", type: "mass", anchor: { x: 0, y: 0, z: 0 }, renderPass: "midMass" } as const;

  assert.equal(computeRenderPassPolicy(mass, -40), "farMass");
  assert.equal(computeRenderPassPolicy(mass, 0), "midMass");
  assert.equal(computeRenderPassPolicy(mass, 40), "nearMass");
});

test("computeRenderPassPolicy respects explicit near/far mass override inside middle threshold", () => {
  const nearMass = { id: "nose", type: "mass", anchor: { x: 0, y: 0, z: 0 }, renderPass: "nearMass" } as const;
  const farMass = { id: "earFar", type: "mass", anchor: { x: 0, y: 0, z: 0 }, renderPass: "farMass" } as const;

  assert.equal(computeRenderPassPolicy(nearMass, 4), "nearMass");
  assert.equal(computeRenderPassPolicy(farMass, -4), "farMass");
});

test("computeRenderPasses keeps far ear before face before near ear before nose", () => {
  const makePart = (
    id: string,
    renderPass: EvaluatedBodyPart["renderPass"],
    zIndex: number,
    depth: number,
  ): EvaluatedBodyPart => ({
    id,
    type: "mass",
    source: { id, type: "mass", anchor: { x: 0, y: 0, z: 0 }, render: { zIndex } },
    projected: { x: 0, y: 0, depth },
    visible: true,
    depth,
    renderPass,
    fusion: { group: "skin", t: 0 },
    geometry: null,
    children: [],
  });

  const sorted = computeRenderPasses([
    makePart("nose", "nearMass", 30, 8),
    makePart("face", "midMass", 20, 0),
    makePart("earNear", "midMass", 25, 4),
    makePart("earFar", "farMass", 25, -6),
  ]);

  assert.deepEqual(sorted.map(item => item.id), ["earFar", "face", "earNear", "nose"]);
});
