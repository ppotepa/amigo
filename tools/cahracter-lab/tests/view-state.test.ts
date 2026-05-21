import test from "node:test";
import assert from "node:assert/strict";

import { computeRigViewState } from "../src/rig";

function stateAt(deg: number) {
  const rad = (deg * Math.PI) / 180;
  return computeRigViewState(rad, deg);
}

test("front and three-quarter views keep nose and mouth visible", () => {
  const front = stateAt(0);
  const angle = stateAt(45);

  assert.equal(front.zone, "FRONT");
  assert.equal(front.showNose, true);
  assert.equal(front.showMouth, true);

  assert.equal(angle.zone, "THREE_QUARTER");
  assert.equal(angle.showNose, true);
  assert.equal(angle.showMouth, true);
});

test("profile keeps nose visible but suppresses mouth late in the turn", () => {
  const profile = stateAt(90);

  assert.equal(profile.zone, "PROFILE");
  assert.equal(profile.showNose, true);
  assert.equal(profile.showMouth, false);
});

test("rear transition and back hide frontal features", () => {
  const rear = stateAt(135);
  const back = stateAt(180);

  assert.equal(rear.zone, "REAR_TRANSITION");
  assert.equal(rear.showNose, false);
  assert.equal(rear.showMouth, false);

  assert.equal(back.zone, "BACK_PROXY");
  assert.equal(back.showNose, false);
  assert.equal(back.showMouth, false);
});

test("rear-side return path keeps rear transition hidden until front-facing states come back", () => {
  const rearQuarter = stateAt(225);
  const profileReturn = stateAt(270);
  const frontQuarter = stateAt(315);

  assert.equal(rearQuarter.zone, "REAR_TRANSITION");
  assert.equal(rearQuarter.showNose, false);
  assert.equal(rearQuarter.showMouth, false);

  assert.equal(profileReturn.zone, "PROFILE");
  assert.equal(profileReturn.showNose, true);
  assert.equal(profileReturn.showMouth, false);

  assert.equal(frontQuarter.zone, "THREE_QUARTER");
  assert.equal(frontQuarter.showNose, true);
  assert.equal(frontQuarter.showMouth, true);
});
