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

export function computeOutlinePolicy(viewState: ViewState, mode: OutlineMode): OutlinePolicy {
  const earPolicies = {
    earRight: computeEarPolicy(viewState.earRight, viewState, mode),
    earLeft: computeEarPolicy(viewState.earLeft, viewState, mode),
  };

  if (mode === OUTLINE_MODES.FULL) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawContour: true, drawSilhouette: true }),
        nose: partPolicy({ drawBody: viewState.showNose, drawContour: viewState.showNose, drawSilhouette: viewState.showNose }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: viewState.showMouth }),
      },
    };
  }
  if (mode === OUTLINE_MODES.SILHOUETTE_ONLY) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawSilhouette: true }),
        nose: partPolicy({ drawBody: false, drawSilhouette: false }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: false }),
      },
    };
  }
  if (mode === OUTLINE_MODES.PAINTERLY) {
    return {
      drawMasterSilhouette: true,
      parts: {
        face: partPolicy({ drawSilhouette: true }),
        nose: partPolicy({
          drawBody: viewState.showNose,
          drawContour: viewState.showNose && viewState.noseFusion < 0.15,
          drawSilhouette: viewState.showNose,
        }),
        earRight: earPolicies.earRight,
        earLeft: earPolicies.earLeft,
        mouth: partPolicy({ drawBody: viewState.zone !== "BACK_PROXY" && viewState.profile < 0.9 }),
      },
    };
  }
  return {
    drawMasterSilhouette: true,
    parts: {
      face: partPolicy({ drawSilhouette: true }),
      nose: partPolicy({
        drawBody: viewState.showNose,
        drawContour: viewState.showNose && viewState.noseFusion < 0.55,
        drawSilhouette: viewState.showNose,
      }),
      earRight: earPolicies.earRight,
      earLeft: earPolicies.earLeft,
      mouth: partPolicy({ drawBody: viewState.zone !== "BACK_PROXY" }),
    },
  };
}
