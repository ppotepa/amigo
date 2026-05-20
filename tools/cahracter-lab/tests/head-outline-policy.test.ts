import test from "node:test";
import assert from "node:assert/strict";

import { computeOutlinePolicy } from "../src/head-outline-policy";
import { OUTLINE_MODES, type EarState, type ViewState } from "../src/types";

function makeEarState(key: EarState["key"], overrides: Partial<EarState> = {}): EarState {
  return {
    key,
    localSide: key === "earRight" ? 1 : -1,
    depth: 0,
    screenX: 0,
    isNear: key === "earRight",
    isFar: key === "earLeft",
    frontLike: true,
    profileT: 0.5,
    backT: 0,
    fusion: 0.2,
    ...overrides,
  };
}

function makeViewState(overrides: Partial<ViewState> = {}): ViewState {
  const earRight = makeEarState("earRight");
  const earLeft = makeEarState("earLeft");
  return {
    yawDeg: 45,
    side: 1,
    profile: 0.5,
    back: 0,
    zone: "THREE_QUARTER",
    t: 0.5,
    noseFusion: 0.2,
    noseMode: "MERGING",
    ears: [earRight, earLeft],
    earRight,
    earLeft,
    nearEar: earRight,
    farEar: earLeft,
    showNose: true,
    showMouth: true,
    ...overrides,
  };
}

test("silhouette-only policy keeps body silhouettes and hides nose body", () => {
  const policy = computeOutlinePolicy(makeViewState(), OUTLINE_MODES.SILHOUETTE_ONLY);

  assert.equal(policy.parts.face.drawContour, false);
  assert.equal(policy.parts.nose.drawBody, false);
  assert.equal(policy.parts.nose.drawSilhouette, false);
});

test("full policy preserves contour on face and visible nose", () => {
  const policy = computeOutlinePolicy(makeViewState(), OUTLINE_MODES.FULL);

  assert.equal(policy.parts.face.drawContour, true);
  assert.equal(policy.parts.face.drawSilhouette, true);
  assert.equal(policy.parts.nose.drawBody, true);
  assert.equal(policy.parts.nose.drawContour, true);
});
