import { TAU, clamp, clamp01, hash01, lerp, noise } from '../math/core.js';
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
  return {
    id: `${kind}:${Math.round(seed * 1000)}`,
    kind,
    d,
    points: contour,
    bounds: boundsForPoints(contour),
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

function pushVisibleRegion(regions, region, frame) {
  if (region && regionInViewport(region, frame)) regions.push(region);
}

function selectRegionGroups(faces, predicate, maxRegions, seedBase) {
  const selected = faces.filter(predicate);
  if (!selected.length) return [];
  const center = regionCenter(selected.map(faceCentroid));
  const buckets = new Map();
  for (const face of selected) {
    const p = faceCentroid(face);
    const angle = Math.atan2(p.y - center.y, p.x - center.x);
    const ring = Math.hypot(p.x - center.x, p.y - center.y) > 140 ? 1 : 0;
    const sector = Math.floor((((angle + Math.PI) / TAU) * maxRegions + hash01(face.id + seedBase) * 0.35) % maxRegions);
    const key = `${sector}:${ring}`;
    if (!buckets.has(key)) buckets.set(key, []);
    buckets.get(key).push(face);
  }
  return [...buckets.values()]
    .filter(group => group.length >= 2)
    .sort((a, b) => b.length - a.length)
    .slice(0, maxRegions);
}

export function buildPaintRegions(frame, state) {
  const faces = frame.paintFaces || [];
  if (!faces.length) return [];
  const style = resolvePaintStyle(state);
  const regionScale = clamp((state.paintRegionResolution || 384) / 384, 0.65, 1.35);
  const base = hexRgb(state.paintBaseColor, '#d7ad85');
  const shadow = hexRgb(state.paintShadowColor, '#5d6f95');
  const highlight = hexRgb(state.paintHighlightColor, '#fff0c2');
  const regions = [];
  const baseRegion = makeRegion(
    'base',
    faces,
    rgbCss(base),
    state.paintBaseOpacity * style.opacity,
    11.31,
    state,
    style,
    'source-over',
  );
  pushVisibleRegion(regions, baseRegion, frame);

  const washGroups = selectRegionGroups(
    faces,
    face => clamp01(face.tone) > 0.16,
    Math.max(2, Math.round((style.shadowRegions - 2) * regionScale)),
    17.9,
  );
  for (let i = 0; i < washGroups.length; i++) {
    const tone = averageTone(washGroups[i]).tone;
    const color = rgbCss(mixRgb(base, shadow, clamp01(tone * 0.82)));
    const region = makeRegion('wash', washGroups[i], color, state.paintWashOpacity * style.washOpacity * (0.22 + tone * 0.48), 21.7 + i, state, style, style.composite);
    pushVisibleRegion(regions, region, frame);
  }

  const steps = Math.max(1, Math.round(state.paintCelSteps || 1));
  const shadowGroups = selectRegionGroups(
    faces,
    face => Math.floor(clamp01(face.tone) * steps) / steps > 0,
    Math.max(2, Math.round(style.shadowRegions * regionScale)),
    31.4,
  );
  for (let i = 0; i < shadowGroups.length; i++) {
    const tone = averageTone(shadowGroups[i]).tone;
    const stepped = Math.floor(clamp01(tone) * steps) / steps;
    const region = makeRegion('shadow', shadowGroups[i], rgbCss(shadow), state.paintCelStrength * (0.28 + stepped * 0.82), 41.2 + i, state, style, 'multiply');
    pushVisibleRegion(regions, region, frame);
  }

  const highlightGroups = selectRegionGroups(
    faces,
    face => clamp01((0.34 - face.tone) * 2.2) > 0.08,
    Math.max(1, Math.round(style.highlightRegions * regionScale)),
    53.1,
  );
  for (let i = 0; i < highlightGroups.length; i++) {
    const tone = averageTone(highlightGroups[i]).tone;
    const light = clamp01((0.34 - tone) * 2.2);
    const region = makeRegion('highlight', highlightGroups[i], rgbCss(highlight), state.paintHighlightAmount * (0.2 + light), 61.8 + i, state, style, 'screen');
    pushVisibleRegion(regions, region, frame);
  }

  return regions;
}
