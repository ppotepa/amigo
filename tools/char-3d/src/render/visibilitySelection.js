import { area2d, boundsCorners, intersectsViewport } from '../scene/bounds.js';
import { projectWorldPoint } from './projectionContext.js';

export function selectVisibleRenderUnits(partition, projectionContext, viewport, state) {
  const units = partition?.units || [];
  const margin = Number(state.visibilityMarginPx) || 0;
  const minArea = Math.max(0, Number(state.visibilityMinAreaPx) || 0);
  const minRadius = Math.max(0, Number(state.visibilityMinRadiusPx) || 0);
  const out = [];
  let culled = 0;
  let skippedBySize = 0;
  let inFrontCount = 0;

  for (const unit of units) {
    const projected = projectUnitBounds(unit, projectionContext);
    if (!projected.inFront) {
      culled++;
      continue;
    }
    inFrontCount++;
    if (state.visibilityCullingEnabled && !intersectsViewport(projected.bounds2d, viewport, margin)) {
      culled++;
      continue;
    }
    if (state.visibilityCullingEnabled && (projected.areaPx < minArea || projected.radiusPx < minRadius)) {
      skippedBySize++;
      continue;
    }
    out.push({
      unit,
      projectedAreaPx: projected.areaPx,
      projectedRadiusPx: projected.radiusPx,
      distance: projected.distance,
      bounds2d: projected.bounds2d,
      detailTier: 0,
    });
  }

  return {
    items: out,
    counters: {
      sceneUnitsTotal: units.length,
      sceneUnitsInFront: inFrontCount,
      sceneUnitsVisible: out.length,
      sceneUnitsCulled: culled,
      sceneUnitsSkippedBySize: skippedBySize,
    },
  };
}

export function projectUnitBounds(unit, projectionContext) {
  const corners = boundsCorners(unit.bounds);
  const bounds2d = { minX: Infinity, minY: Infinity, maxX: -Infinity, maxY: -Infinity };
  let inFront = false;
  let zSum = 0;
  let zCount = 0;
  const tmp = {};
  for (const c of corners) {
    projectWorldPoint(projectionContext, c[0], c[1], c[2], -1, tmp);
    if (tmp.inFront) inFront = true;
    bounds2d.minX = Math.min(bounds2d.minX, tmp.sx);
    bounds2d.minY = Math.min(bounds2d.minY, tmp.sy);
    bounds2d.maxX = Math.max(bounds2d.maxX, tmp.sx);
    bounds2d.maxY = Math.max(bounds2d.maxY, tmp.sy);
    zSum += tmp.z;
    zCount++;
  }
  if (!Number.isFinite(bounds2d.minX)) {
    bounds2d.minX = bounds2d.minY = bounds2d.maxX = bounds2d.maxY = 0;
  }
  const w = Math.max(0, bounds2d.maxX - bounds2d.minX);
  const h = Math.max(0, bounds2d.maxY - bounds2d.minY);
  return {
    inFront,
    bounds2d,
    areaPx: area2d(bounds2d),
    radiusPx: Math.hypot(w, h) * 0.5,
    distance: zSum / Math.max(1, zCount),
  };
}
