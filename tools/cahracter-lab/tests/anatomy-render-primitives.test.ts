import test from "node:test";
import assert from "node:assert/strict";

import { generateAnatomyRenderPrimitives } from "../src/anatomy-render-primitives";
import { type EvaluatedBodyPart, type OutlinePolicy } from "../src/types";

function makeMassPart(id: string, zIndex: number, path = `M 0 0 L 1 1`): EvaluatedBodyPart {
  return {
    id,
    type: "mass",
    source: {
      id,
      type: "mass",
      anchor: { x: 0, y: 0, z: 0 },
      renderPass: "midMass",
      render: {
        zIndex,
        outline: {
          contourClassName: "partOutline",
          contourZIndex: zIndex + 100,
          silhouetteZIndex: zIndex + 90,
        },
        outputs: [{ kind: "path", role: "fill", className: "surfaceBase" }],
      },
    },
    projected: { x: 0, y: 0, depth: 0 },
    visible: true,
    depth: 0,
    renderPass: "midMass",
    fusion: { group: "skin", t: 0 },
    geometry: { path, visible: true },
    children: [],
  };
}

test("generateAnatomyRenderPrimitives emits body and outline primitives from policy and render hints", () => {
  const parts = [makeMassPart("earLeft", 4), makeMassPart("face", 20)];
  const policy: OutlinePolicy = {
    drawMasterSilhouette: true,
    parts: {
      earLeft: { drawBody: true, drawContour: false, drawInner: false, drawSilhouette: true },
      face: { drawBody: true, drawContour: true, drawInner: false, drawSilhouette: true },
    },
  };

  const primitives = generateAnatomyRenderPrimitives(parts, policy);
  const ids = primitives.map(item => item.id);

  assert.deepEqual(ids, [
    "earLeft:fill:0",
    "face:fill:0",
    "outline:earLeft",
    "silhouette:earLeft",
    "outline:face",
    "silhouette:face",
  ]);

  const faceOutline = primitives.find(item => item.id === "outline:face");
  const earOutline = primitives.find(item => item.id === "outline:earLeft");
  const earSilhouette = primitives.find(item => item.id === "silhouette:earLeft");

  assert.equal(faceOutline?.visible, true);
  assert.equal(earOutline?.visible, false);
  assert.equal(earSilhouette?.visible, true);
});

test("generateAnatomyRenderPrimitives hides body fill and outlines when policy or part visibility disables them", () => {
  const hiddenEar = {
    ...makeMassPart("earRight", 6),
    visible: false,
  };
  const nose = makeMassPart("nose", 10);
  const policy: OutlinePolicy = {
    drawMasterSilhouette: true,
    parts: {
      earRight: { drawBody: true, drawContour: true, drawInner: false, drawSilhouette: true },
      nose: { drawBody: false, drawContour: false, drawInner: false, drawSilhouette: true },
    },
  };

  const primitives = generateAnatomyRenderPrimitives([hiddenEar, nose], policy);
  const hiddenEarFill = primitives.find(item => item.id === "earRight:fill:0");
  const hiddenEarSilhouette = primitives.find(item => item.id === "silhouette:earRight");
  const hiddenNoseFill = primitives.find(item => item.id === "nose:fill:0");
  const visibleNoseSilhouette = primitives.find(item => item.id === "silhouette:nose");

  assert.equal(hiddenEarFill?.visible, false);
  assert.equal(hiddenEarSilhouette?.visible, false);
  assert.equal(hiddenNoseFill?.visible, false);
  assert.equal(visibleNoseSilhouette?.visible, true);
});
