export function assignDetailTiers(visibilityResult, state) {
  const items = visibilityResult.items || visibilityResult || [];
  const counters = {
    detailTier0Units: 0,
    detailTier1Units: 0,
    detailTier2Units: 0,
    detailTier3Units: 0,
    detailTier4Units: 0,
  };

  for (const item of items) {
    const density = Math.max(0, item.unit?.densityEstimate || 0);
    const importance = Math.max(0.05, item.unit?.importance || 1);
    const densityPenalty = Math.max(0, Number(state.detailDensityPenalty) || 0);
    const importanceBias = Math.max(0.05, Number(state.detailImportanceBias) || 1);
    const adjustedRadius = item.projectedRadiusPx
      * Math.pow(importance, importanceBias)
      / (1 + density * densityPenalty);
    item.adjustedRadiusPx = adjustedRadius;
    item.detailTier = state.detailPolicyEnabled ? chooseDetailTier(adjustedRadius, state) : 0;
    counters[`detailTier${item.detailTier}Units`] = (counters[`detailTier${item.detailTier}Units`] || 0) + 1;
  }

  return { items, counters };
}

export function chooseDetailTier(radiusPx, state) {
  if (radiusPx >= Number(state.detailTier0RadiusPx || 180)) return 0;
  if (radiusPx >= Number(state.detailTier1RadiusPx || 80)) return 1;
  if (radiusPx >= Number(state.detailTier2RadiusPx || 28)) return 2;
  if (radiusPx >= Number(state.detailTier3RadiusPx || 8)) return 3;
  return 4;
}

export function detailMarkMultiplier(tier) {
  if (tier <= 0) return 1.0;
  if (tier === 1) return 0.55;
  if (tier === 2) return 0.18;
  if (tier === 3) return 0.04;
  return 0;
}

export function detailAllowsInternalLine(tier, kind) {
  if (kind === 'contour') return tier < 4;
  if (tier <= 1) return true;
  return false;
}

export function detailAllowsRegionFill(tier, state) {
  if (!state.regionBudgetEnabled) return true;
  if (state.regionAllowFarFills) return tier <= 3;
  return tier <= 2;
}
