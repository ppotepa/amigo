import { clamp, clamp01, hash01, lerp, noise } from '../math/core.js';
import { hexRgb, mixRgb } from '../math/color.js';
import { resolvePaintStyle } from './paintStyles.js';

function rgbCss(color) {
  return `rgb(${color.r},${color.g},${color.b})`;
}

function faceCentroid(face) {
  return {
    x: (face.p[0].sx + face.p[1].sx + face.p[2].sx) / 3,
    y: (face.p[0].sy + face.p[1].sy + face.p[2].sy) / 3,
  };
}

function regionCenter(points) {
  const out = { x: 0, y: 0 };
  for (const p of points) {
    out.x += p.x;
    out.y += p.y;
  }
  const inv = 1 / Math.max(1, points.length);
  out.x *= inv;
  out.y *= inv;
  return out;
}

function cross(o, a, b) {
  return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
}

function convexHull(points) {
  if (points.length <= 3) return points.slice();
  const sorted = points.slice().sort((a, b) => a.x === b.x ? a.y - b.y : a.x - b.x);
  const lower = [];
  for (const p of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) lower.pop();
    lower.push(p);
  }
  const upper = [];
  for (let i = sorted.length - 1; i >= 0; i--) {
    const p = sorted[i];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) upper.pop();
    upper.push(p);
  }
  lower.pop();
  upper.pop();
  return lower.concat(upper);
}

function simplifyPoints(points, simplify) {
  if (points.length <= 14) return points;
  const stride = clamp(Math.round(1 + simplify * 5), 1, 6);
  const out = [];
  for (let i = 0; i < points.length; i += stride) out.push(points[i]);
  return out.length >= 5 ? out : points.slice(0, 18);
}

function organicContour(points, seed, style, simplify) {
  let hull = simplifyPoints(convexHull(points), simplify);
  if (hull.length < 3) return null;
  const center = regionCenter(hull);
  const out = [];
  for (let i = 0; i < hull.length; i++) {
    const p = hull[i];
    const next = hull[(i + 1) % hull.length];
    const dx = p.x - center.x;
    const dy = p.y - center.y;
    const len = Math.max(0.001, Math.hypot(dx, dy));
    const amp = style.jitter * lerp(2.5, 11, hash01(seed + i * 9.13));
    out.push({
      x: p.x + (dx / len) * noise(seed, i * 2 + 1) * amp,
      y: p.y + (dy / len) * noise(seed, i * 2 + 2) * amp,
    });
    const mx = (p.x + next.x) * 0.5;
    const my = (p.y + next.y) * 0.5;
    const mdx = mx - center.x;
    const mdy = my - center.y;
    const mlen = Math.max(0.001, Math.hypot(mdx, mdy));
    out.push({
      x: mx + (mdx / mlen) * noise(seed, i * 2 + 5) * amp * 0.75,
      y: my + (mdy / mlen) * noise(seed, i * 2 + 6) * amp * 0.75,
    });
  }
  return out;
}

export function smoothRegionPath(points) {
  if (!points || points.length < 3) return '';
  const first = points[0];
  let d = `M ${first.x.toFixed(1)} ${first.y.toFixed(1)}`;
  for (let i = 1; i <= points.length; i++) {
    const p = points[i % points.length];
    const next = points[(i + 1) % points.length];
    const mx = (p.x + next.x) * 0.5;
    const my = (p.y + next.y) * 0.5;
    d += ` Q ${p.x.toFixed(1)} ${p.y.toFixed(1)} ${mx.toFixed(1)} ${my.toFixed(1)}`;
  }
  return `${d} Z`;
}

function collectFacePoints(faces, includeVertices = true) {
  const points = [];
  for (const face of faces) {
    points.push(faceCentroid(face));
    if (includeVertices) {
      points.push({ x: face.p[0].sx, y: face.p[0].sy });
      points.push({ x: face.p[1].sx, y: face.p[1].sy });
      points.push({ x: face.p[2].sx, y: face.p[2].sy });
    }
  }
  return points;
}

function averageTone(faces) {
  let tone = 0;
  let visibility = 0;
  let depth = 0;
  for (const face of faces) {
    tone += clamp01(face.tone);
    visibility += clamp01(face.visibility);
    depth += face.depth || 0;
  }
  const inv = 1 / Math.max(1, faces.length);
  return { tone: tone * inv, visibility: visibility * inv, depth: depth * inv };
}

function boundsArea(bounds) {
  if (!bounds) return 0;
  return Math.max(0, bounds.w || 0) * Math.max(0, bounds.h || 0);
}

function averageDetailTier(faces) {
  if (!faces.length) return 0;
  let sum = 0;
  for (const face of faces) sum += face.detailTier ?? 0;
  return sum / faces.length;
}

function regionSamples(faces, seed, maxSamples = 18) {
  if (!faces.length) return [];
  const stride = Math.max(1, Math.floor(faces.length / maxSamples));
  const out = [];
  for (let i = 0; i < faces.length && out.length < maxSamples; i += stride) {
    const face = faces[(i + Math.floor(hash01(seed + i) * stride)) % faces.length];
    out.push({
      x: face.cx,
      y: face.cy,
      z: face.depth || 0,
      tone: clamp01(face.tone),
      visibility: clamp01(face.visibility),
      flow: face.flow || { x: 1, y: 0 },
      area: face.area || 0,
    });
  }
  return out;
}

function makeRegion(kind, faces, color, opacity, seed, state, style, composite = 'source-over') {
  const points = collectFacePoints(faces);
  const contour = organicContour(points, seed, style, clamp01(state.paintRegionSimplify ?? 0.45));
  if (!contour) return null;
  const tone = averageTone(faces);
  const d = smoothRegionPath(contour);
  const bounds = boundsForPoints(contour);
  return {
    id: `${kind}:${Math.round(seed * 1000)}`,
    kind,
    d,
    points: contour,
    bounds,
    projectedAreaPx: boundsArea(bounds),
    detailTier: averageDetailTier(faces),
    faceCount: faces.length,
    color,
    opacity: clamp01(opacity * tone.visibility),
    composite,
    blur: style.edgeSoftness * (kind === 'base' ? 0.35 : 1),
    grain: style.granulation,
    bleed: style.bleed,
    seed,
    tone: tone.tone,
    depth: tone.depth,
    samples: regionSamples(faces, seed),
  };
}

function boundsForPoints(points) {
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const p of points) {
    minX = Math.min(minX, p.x);
    minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x);
    maxY = Math.max(maxY, p.y);
  }
  return { minX, minY, maxX, maxY, w: maxX - minX, h: maxY - minY };
}

function faceVertexKey(vertex) {
  if (!vertex) return '';
  if (Number.isFinite(vertex._paintRegionVertexKey)) return vertex._paintRegionVertexKey;
  return `${Math.round(vertex.sx * 10)}:${Math.round(vertex.sy * 10)}:${Math.round((vertex.z || 0) * 1000)}`;
}

function faceAreaSum(faces) {
  let area = 0;
  for (const face of faces) area += Math.abs(face.area || 0);
  return area;
}

function groupAspect(faces) {
  const bounds = boundsForPoints(collectFacePoints(faces, false));
  const shortSide = Math.max(1, Math.min(bounds.w, bounds.h));
  return Math.max(bounds.w, bounds.h) / shortSide;
}

function regionInViewport(region, frame) {
  const viewport = frame?.viewport;
  if (!viewport || !region?.bounds) return true;
  const margin = Math.max(0, viewport.margin || 0);
  const b = region.bounds;
  return b.maxX >= -margin
    && b.maxY >= -margin
    && b.minX <= viewport.width + margin
    && b.minY <= viewport.height + margin;
}

function pushVisibleRegion(regions, region, frame, state) {
  if (!region || !regionInViewport(region, frame)) return;
  if (state?.regionBudgetEnabled) {
    if ((region.projectedAreaPx || 0) < (Number(state.regionMinProjectedAreaPx) || 0)) return;
    if (!state.regionAllowFarFills && (region.detailTier || 0) > 2) return;
    if (regions.length >= Math.max(0, Number(state.regionMaxPaintRegions) || 0)) return;
  }
  regions.push(region);
}

function regionSetEnabled(state, id) {
  const flat = `${id}Enabled`;
  if (flat in state) return !!state[flat];
  return state.regionSets?.[id]?.enabled !== false;
}

function splitConnectedFaceGroups(selected) {
  if (selected.length <= 1) return selected.length ? [selected] : [];
  const vertexToFaces = new Map();
  for (let i = 0; i < selected.length; i++) {
    const face = selected[i];
    for (const vertex of face.p || []) {
      const key = faceVertexKey(vertex);
      if (!vertexToFaces.has(key)) vertexToFaces.set(key, []);
      vertexToFaces.get(key).push(i);
    }
  }

  const visited = new Uint8Array(selected.length);
  const groups = [];
  for (let start = 0; start < selected.length; start++) {
    if (visited[start]) continue;
    const group = [];
    const stack = [start];
    visited[start] = 1;
    while (stack.length) {
      const index = stack.pop();
      const face = selected[index];
      group.push(face);
      for (const vertex of face.p || []) {
        const neighbors = vertexToFaces.get(faceVertexKey(vertex)) || [];
        for (const next of neighbors) {
          if (visited[next]) continue;
          visited[next] = 1;
          stack.push(next);
        }
      }
    }
    groups.push(group);
  }
  return groups;
}

function regionGroupAccepted(group, state) {
  const area = faceAreaSum(group);
  const minArea = state.cleanupRegionMinAreaPx ?? 80;
  const minFaces = Math.max(1, Math.round(state.cleanupRegionMinFaces ?? 3));
  if (group.length < minFaces || area < minArea) return false;
  const aspect = groupAspect(group);
  if (aspect > (state.cleanupRegionMaxAspect ?? 16)) return false;

  const avgFaceArea = area / Math.max(1, group.length);
  const hairLike = avgFaceArea < Math.max(2, minArea * 0.025) && aspect > 5;
  if (hairLike && (state.hairRegionSuppression ?? 0.5) > 0) {
    return hash01(group[0].id + area * 0.013) > clamp01(state.hairRegionSuppression ?? 0.5);
  }
  return true;
}

function selectRegionGroups(faces, predicate, maxRegions, seedBase, state) {
  const selected = faces.filter(predicate);
  if (!selected.length) return [];
  return splitConnectedFaceGroups(selected)
    .filter(group => regionGroupAccepted(group, state))
    .sort((a, b) => b.length - a.length)
    .slice(0, maxRegions);
}

export function buildPaintRegions(frame, state) {
  const faces = frame.paintFaces || [];
  if (!faces.length) return [];
  const sourceFaces = state.regionBudgetEnabled
    ? faces.filter(face => (face.detailTier ?? 0) <= (state.regionAllowFarFills ? 3 : 2) && Math.abs(face.area || 0) >= (Number(state.vectorMinFaceAreaPx) || 0))
    : faces;
  if (!sourceFaces.length) return [];
  const style = resolvePaintStyle(state);
  const regionScale = clamp((state.paintRegionResolution || 384) / 384, 0.65, 1.35);
  const base = hexRgb(state.paintBaseColor, '#d7ad85');
  const shadow = hexRgb(state.paintShadowColor, '#5d6f95');
  const highlight = hexRgb(state.paintHighlightColor, '#fff0c2');
  const regions = [];
  if (regionSetEnabled(state, 'baseWash')) {
    const baseRegion = makeRegion(
      'base',
      sourceFaces,
      rgbCss(base),
      state.paintBaseOpacity * style.opacity,
      11.31,
      state,
      style,
      'source-over',
    );
    pushVisibleRegion(regions, baseRegion, frame, state);
  }

  const washGroups = selectRegionGroups(
    sourceFaces,
    face => clamp01(face.tone) > 0.16,
    Math.max(2, Math.round((style.shadowRegions - 2) * regionScale)),
    17.9,
    state,
  );
  for (let i = 0; i < washGroups.length; i++) {
    const tone = averageTone(washGroups[i]).tone;
    const color = rgbCss(mixRgb(base, shadow, clamp01(tone * 0.82)));
    const region = makeRegion('wash', washGroups[i], color, state.paintWashOpacity * style.washOpacity * (0.22 + tone * 0.48), 21.7 + i, state, style, style.composite);
    pushVisibleRegion(regions, region, frame, state);
  }

  const steps = Math.max(1, Math.round(state.paintCelSteps || 1));
  const bandCount = Math.max(1, Math.round(state.shadowBandCount || steps));
  const shadowGroups = selectRegionGroups(
    sourceFaces,
    face => Math.floor(clamp01(face.tone) * bandCount) / bandCount > 0,
    Math.max(2, Math.round(style.shadowRegions * regionScale)),
    31.4,
    state,
  );
  if (regionSetEnabled(state, 'shadowRegion')) {
    for (let i = 0; i < shadowGroups.length; i++) {
      const tone = averageTone(shadowGroups[i]).tone;
      const stepped = Math.floor(clamp01(tone) * bandCount) / bandCount;
      const jitteredShadow = mixRgb(shadow, base, clamp01(noise(41.2 + i, 6) * (state.shadowColorJitter || 0) * .25));
      const region = makeRegion('shadow', shadowGroups[i], rgbCss(jitteredShadow), state.paintCelStrength * (0.28 + stepped * 0.82), 41.2 + i, state, {...style, bleed: style.bleed + (state.shadowRegionBleed || 0) * .25}, 'multiply');
      pushVisibleRegion(regions, region, frame, state);
    }
  }

  const highlightGroups = selectRegionGroups(
    sourceFaces,
    face => clamp01((0.34 - face.tone) * 2.2) > 0.08,
    Math.max(1, Math.round(style.highlightRegions * regionScale)),
    53.1,
    state,
  );
  if (regionSetEnabled(state, 'highlightRegion')) {
    for (let i = 0; i < highlightGroups.length; i++) {
      const tone = averageTone(highlightGroups[i]).tone;
      const light = clamp01((0.34 - tone) * 2.2);
      const region = makeRegion('highlight', highlightGroups[i], rgbCss(highlight), state.paintHighlightAmount * (0.2 + light), 61.8 + i, state, style, 'screen');
      pushVisibleRegion(regions, region, frame, state);
    }
  }

  return regions;
}
