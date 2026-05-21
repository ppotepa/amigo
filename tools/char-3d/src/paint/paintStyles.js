import { clamp01, lerp } from '../math/core.js';

const BRUSHES = {
  watercolor: {
    edgeSoftness: 1.25,
    bleed: 1.15,
    opacity: 0.88,
    washOpacity: 1.18,
    pigment: 1.2,
    granulation: 1.35,
    jitter: 1.1,
    shadowRegions: 7,
    highlightRegions: 4,
    composite: 'multiply',
  },
  gouache: {
    edgeSoftness: 0.45,
    bleed: 0.42,
    opacity: 1.08,
    washOpacity: 0.72,
    pigment: 0.62,
    granulation: 0.45,
    jitter: 0.55,
    shadowRegions: 6,
    highlightRegions: 5,
    composite: 'source-over',
  },
  comicCel: {
    edgeSoftness: 0.14,
    bleed: 0.08,
    opacity: 1.18,
    washOpacity: 0.25,
    pigment: 0.18,
    granulation: 0.12,
    jitter: 0.18,
    shadowRegions: 4,
    highlightRegions: 3,
    composite: 'multiply',
  },
  inkWash: {
    edgeSoftness: 0.9,
    bleed: 0.8,
    opacity: 0.72,
    washOpacity: 1.35,
    pigment: 1.45,
    granulation: 1.1,
    jitter: 0.85,
    shadowRegions: 8,
    highlightRegions: 2,
    composite: 'multiply',
  },
};

export function paintBrushOptions() {
  return Object.keys(BRUSHES);
}

export function resolvePaintStyle(state) {
  const brush = BRUSHES[state.paintBrush] ? state.paintBrush : 'watercolor';
  const base = BRUSHES[brush];
  const bleed = clamp01(state.paintEdgeBleed ?? ((state.paintBleed ?? 0) / 5));
  const granulation = clamp01(state.paintPigmentGranulation ?? state.paintGrain ?? 0);
  const jitter = clamp01(state.paintRegionJitter ?? 0);
  const wet = clamp01(state.paintWetMix ?? 0);

  return {
    id: brush,
    edgeSoftness: base.edgeSoftness * lerp(0.45, 1.35, bleed),
    bleed: base.bleed * lerp(0.4, 1.55, bleed),
    opacity: base.opacity,
    washOpacity: base.washOpacity * lerp(0.75, 1.25, wet),
    pigment: base.pigment,
    granulation: base.granulation * lerp(0.45, 1.55, granulation),
    jitter: base.jitter * lerp(0.35, 1.65, jitter),
    shadowRegions: base.shadowRegions,
    highlightRegions: base.highlightRegions,
    composite: base.composite,
  };
}
