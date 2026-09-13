import { remapClamp, smoothstep } from "./math";
import { OUTLINE_MODES, type EarState, type OutlineMode, type OutlinePartPolicy, type OutlinePolicy, type ViewState } from "./types";

function computeEarPolicy(earState: EarState, viewState: ViewState, mode: OutlineMode) {
  const isProfileFar = earState.isFar && viewState.profile > 0.64;
  const isBackSide = viewState.back > 0.62;
  if (mode === OUTLINE_MODES.FULL) {
    return { drawBody: true, drawContour: true, drawInner: true, drawSilhouette: true };
  }
  if (mode === OUTLINE_MODES.SILHOUETTE_ONLY) {
    return { drawBody: true, drawContour: false, drawInner: false, drawSilhouette: !isProfileFar };
  }
  if (mode === OUTLINE_MODES.PAINTERLY) {
    return {
      drawBody: true,
      drawContour: earState.isNear && viewState.profile < 0.38 && !isBackSide,
      drawInner: false,
      drawSilhouette: !isProfileFar,
    };
  }
  return {
    drawBody: true,
    drawContour: earState.isNear ? viewState.profile < 0.86 || isBackSide : viewState.profile < 0.36 || isBackSide,
    drawInner: earState.isNear && viewState.profile < 0.68 && viewState.back < 0.58,
    drawSilhouette: !isProfileFar,
  };
}

function partPolicy(overrides: Partial<OutlinePartPolicy> = {}): OutlinePartPolicy {
  return {
    drawBody: true,
    drawContour: false,
    drawInner: false,
    drawSilhouette: false,
    ...overrides,
  };
}

function noseFacingScore(viewState: ViewState): number {
  const profilePresence = smoothstep(remapClamp(viewState.profile, 0.18, 0.82));
  const backFade = 1 - smoothstep(remapClamp(viewState.back, 0.12, 0.72));
  return profilePresence * backFade;
}

export function computeOutlinePolicy(viewState: ViewState, mode: OutlineMode): OutlinePolicy {
  const showFace = true;
  const showNose = viewState.showNose;
  const showMouth = viewState.showMouth;
  const showEarInner = viewState.zone !== "REAR_TRANSITION" && viewState.zone !== "BACK_PROXY" && viewState.profile < 0.72;
  const showNoseDetail = viewState.zone === "FRONT" || viewState.zone === "THREE_QUARTER" || (viewState.zone === "PROFILE" && viewState.back < 0.08);
  const showNoseDetailStrong = showNoseDetail && noseFacingScore(viewState) > 0.5;
  const adaptiveNoseDetail = showNoseDetail && noseFacingScore(viewState) > 0.12;
  const earPolicies = {
    earRight: computeEarPolicy(viewState.earRight, viewState, mode),
    earLeft: computeEarPolicy(viewState.earLeft, viewState, mode),
  };

  if (mode === OUTLINE_MODES.FULL) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawBody: showFace, drawContour: true, drawSilhouette: true }),
        nose: partPolicy({ drawBody: showNose, drawContour: showNose, drawSilhouette: showNose }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: showMouth, drawContour: false }),
        noseHighlight: partPolicy({ drawBody: showNose }),
        nostril: partPolicy({ drawBody: showNose }),
        earRightInner: partPolicy({ drawBody: earPolicies.earRight.drawInner && showEarInner }),
        earLeftInner: partPolicy({ drawBody: earPolicies.earLeft.drawInner && showEarInner }),
      },
    };
  }
  if (mode === OUTLINE_MODES.SILHOUETTE_ONLY) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
        nose: partPolicy({ drawBody: showNose, drawSilhouette: showNose }),
        earRight: partPolicy({ drawBody: earPolicies.earRight.drawBody, drawSilhouette: earPolicies.earRight.drawSilhouette }),
        earLeft: partPolicy({ drawBody: earPolicies.earLeft.drawBody, drawSilhouette: earPolicies.earLeft.drawSilhouette }),
        mouth: partPolicy({ drawBody: false }),
        noseHighlight: partPolicy({ drawBody: false }),
        nostril: partPolicy({ drawBody: false }),
        earRightInner: partPolicy({ drawBody: false }),
        earLeftInner: partPolicy({ drawBody: false }),
      },
    };
  }
  if (mode === OUTLINE_MODES.PAINTERLY) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
        nose: partPolicy({
          drawBody: showNose,
          drawContour: showNose && viewState.noseFusion < 0.18 && viewState.zone !== "PROFILE",
          drawSilhouette: showNose,
        }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: showMouth && viewState.profile < 0.82 }),
        noseHighlight: partPolicy({ drawBody: showNoseDetailStrong }),
        nostril: partPolicy({ drawBody: showNoseDetailStrong }),
        earRightInner: partPolicy({ drawBody: false }),
        earLeftInner: partPolicy({ drawBody: false }),
      },
    };
  }
  return {
    drawMasterSilhouette: true,
    parts: {
      face: partPolicy({ drawBody: showFace, drawSilhouette: true }),
      nose: partPolicy({
        drawBody: showNose,
        drawContour: showNose && viewState.noseFusion < 0.55 && viewState.zone !== "REAR_TRANSITION",
        drawSilhouette: showNose,
      }),
      earRight: earPolicies.earRight,
      earLeft: earPolicies.earLeft,
      mouth: partPolicy({ drawBody: showMouth }),
      noseHighlight: partPolicy({ drawBody: adaptiveNoseDetail }),
      nostril: partPolicy({ drawBody: adaptiveNoseDetail }),
      earRightInner: partPolicy({ drawBody: earPolicies.earRight.drawInner && showEarInner }),
      earLeftInner: partPolicy({ drawBody: earPolicies.earLeft.drawInner && showEarInner }),
    },
  };
}
