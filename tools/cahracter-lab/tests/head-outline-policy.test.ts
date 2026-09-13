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

test("silhouette-only policy keeps one silhouette set for visible masses and hides inner details", () => {
  const policy = computeOutlinePolicy(makeViewState(), OUTLINE_MODES.SILHOUETTE_ONLY);

  assert.equal(policy.parts.face.drawContour, false);
  assert.equal(policy.parts.nose.drawBody, true);
  assert.equal(policy.parts.nose.drawSilhouette, true);
  assert.equal(policy.parts.earRight.drawContour, false);
  assert.equal(policy.parts.noseHighlight.drawBody, false);
  assert.equal(policy.parts.earRightInner.drawBody, false);
});

test("full policy preserves contour on face and visible nose", () => {
  const policy = computeOutlinePolicy(makeViewState(), OUTLINE_MODES.FULL);

  assert.equal(policy.parts.face.drawContour, true);
  assert.equal(policy.parts.face.drawSilhouette, true);
  assert.equal(policy.parts.nose.drawBody, true);
  assert.equal(policy.parts.nose.drawContour, true);
});

test("adaptive policy keeps nose body visible while suppressing contour and detail by facing", () => {
  const fusedBody = computeOutlinePolicy(makeViewState({ noseFusion: 0.94 }), OUTLINE_MODES.ADAPTIVE);
  const hiddenDetail = computeOutlinePolicy(makeViewState({ profile: 0.12, zone: "FRONT" }), OUTLINE_MODES.ADAPTIVE);
  const visibleDetail = computeOutlinePolicy(makeViewState({ profile: 0.5, zone: "THREE_QUARTER" }), OUTLINE_MODES.ADAPTIVE);
  const rearTransition = computeOutlinePolicy(
    makeViewState({ back: 0.6, zone: "REAR_TRANSITION", showNose: false, noseMode: "HIDDEN" }),
    OUTLINE_MODES.ADAPTIVE,
  );

  assert.equal(fusedBody.parts.nose.drawBody, true);
  assert.equal(fusedBody.parts.nose.drawContour, false);
  assert.equal(hiddenDetail.parts.noseHighlight.drawBody, false);
  assert.equal(visibleDetail.parts.noseHighlight.drawBody, true);
  assert.equal(visibleDetail.parts.nostril.drawBody, true);
  assert.equal(rearTransition.parts.nose.drawBody, false);
  assert.equal(rearTransition.parts.nose.drawContour, false);
});

test("painterly policy keeps nose mass while limiting strong nose detail", () => {
  const fusedBody = computeOutlinePolicy(makeViewState({ noseFusion: 0.5 }), OUTLINE_MODES.PAINTERLY);
  const weakDetail = computeOutlinePolicy(makeViewState({ profile: 0.45 }), OUTLINE_MODES.PAINTERLY);
  const strongDetail = computeOutlinePolicy(makeViewState({ profile: 0.7 }), OUTLINE_MODES.PAINTERLY);
  const profileMouth = computeOutlinePolicy(makeViewState({ profile: 0.9, zone: "PROFILE", showMouth: false }), OUTLINE_MODES.PAINTERLY);

  assert.equal(fusedBody.parts.nose.drawBody, true);
  assert.equal(fusedBody.parts.nose.drawContour, false);
  assert.equal(weakDetail.parts.noseHighlight.drawBody, false);
  assert.equal(strongDetail.parts.noseHighlight.drawBody, true);
  assert.equal(profileMouth.parts.mouth.drawBody, false);
});
