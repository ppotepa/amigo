import { EPS, clamp, clamp01, deg, lerp, noise } from '../math/core.js';
import { buildScenePartition, getScenePartitionKey } from '../scene/scenePartition.js';
import { createProjectionContext, projectWorldPoint } from './projectionContext.js';
import { selectVisibleRenderUnits } from './visibilitySelection.js';
import { assignDetailTiers, detailAllowsInternalLine } from './detailPolicy.js';
import { buildRenderSelection, buildFullRenderSelection } from './renderSelection.js';

let mesh = null;
let meshKey = '';
let partitionCache = { key: '', value: null };

self.onmessage = event => {
  const msg = event.data || {};
  if (msg.type === 'mesh') {
    meshKey = msg.meshKey || '';
    mesh = msg.mesh;
    partitionCache = { key: '', value: null };
    self.postMessage({ type: 'mesh-ready', meshKey });
    return;
  }
  if (msg.type !== 'frame') return;
  const t0 = performance.now();
  try {
    if (!mesh || msg.meshKey !== meshKey) throw new Error('frame worker mesh is not ready');
    const result = computeFrame(msg.params);
    result.timings.workerTotal = performance.now() - t0;
    self.postMessage(
      { type: 'frame', jobId: msg.jobId, meshKey, result },
      transferFrameResult(result)
    );
  } catch (error) {
    self.postMessage({ type: 'error', jobId: msg.jobId, meshKey, message: error?.message || String(error) });
  }
};

function transferFrameResult(result) {
  const out = [
    result.verts.x.buffer, result.verts.y.buffer, result.verts.z.buffer,
    result.verts.sx.buffer, result.verts.sy.buffer, result.verts.inFront.buffer,
    result.db.depth.buffer, result.db.owner.buffer,
  ];
  if (result.verts.localToGlobal?.buffer) out.push(result.verts.localToGlobal.buffer);
  for (const value of Object.values(result.screen)) if (value?.buffer) out.push(value.buffer);
  for (const key of ['x1', 'y1', 'z1', 'x2', 'y2', 'z2', 'kind', 'visible', 'detailTier']) {
    out.push(result.contours[key].buffer);
  }
  return out;
}

function computeFrame(p) {
  const timings = {};
  const selectionStart = performance.now();
  const projectionContext = createProjectionContext({
    ...p,
    centerX: p.width / 2,
    centerY: p.height / 2,
    sourceScaleMul: 1,
  });
  const renderSelection = buildWorkerRenderSelection(p, projectionContext);
  timings.selection = performance.now() - selectionStart;

  const projectionStart = performance.now();
  const verts = projectVertices(p, renderSelection, projectionContext);
  const faceData = buildFaces(p, verts, renderSelection);
  timings.projection = performance.now() - projectionStart;

  const depthStart = performance.now();
  const db = buildDepthBuffer(p, verts, faceData);
  timings.depth = performance.now() - depthStart;

  const visibilityStart = performance.now();
  computeVisibilityAndFlow(p, faceData, db);
  timings.visibility = performance.now() - visibilityStart;

  const contourStart = performance.now();
  const contours = p.contours ? computeContours(p, verts, faceData, db) : emptyContours();
  timings.contours = performance.now() - contourStart;

  return {
    verts,
    screen: packScreenFaces(faceData),
    contours,
    db,
    timings,
    counters: {
      facesTotal: mesh.faceCount,
      facesSelected: renderSelection.faceIds.length,
      vertsSelected: renderSelection.vertexIds.length,
      edgesSelected: renderSelection.edgeIds.length,
      facesOnScreen: faceData.count,
      facesDepth: db.rasterized,
      facesVisible: faceData.visibleCount,
      contoursTested: contours.tested,
      contoursDrawn: contours.count,
      ...renderSelection.counters,
      ...(renderSelection.visibilityCounters || {}),
      ...(renderSelection.detailCounters || {}),
    },
    depthMode: p.controlMode === 'freelook' ? 'min' : 'max',
    viewport: { width: p.width, height: p.height, margin: p.cullMargin },
    L: lightVector(p),
  };
}

function workerRuntime() {
  return mesh;
}

function getOrBuildPartition(p) {
  const runtime = workerRuntime();
  const key = getScenePartitionKey(runtime, p);
  if (partitionCache.key === key && partitionCache.value) return partitionCache.value;
  const value = buildScenePartition(runtime, p);
  partitionCache = { key, value };
  return value;
}

function buildWorkerRenderSelection(p, projectionContext) {
  const runtime = workerRuntime();
  if (!p.scenePartitionEnabled && !p.visibilityCullingEnabled && !p.detailPolicyEnabled && !p.vectorBudgetEnabled) {
    return buildFullRenderSelection(runtime);
  }
  const partition = getOrBuildPartition(p);
  const viewport = { width: p.width, height: p.height };
  const visibility = selectVisibleRenderUnits(partition, projectionContext, viewport, p);
  const detailed = assignDetailTiers(visibility, p);
  const selection = buildRenderSelection(detailed, runtime, p);
  selection.visibilityCounters = visibility.counters;
  selection.detailCounters = detailed.counters;
  return selection;
}

function projectVertices(p, renderSelection, projectionContext) {
  const n = renderSelection.vertexIds.length;
  const x = new Float32Array(n);
  const y = new Float32Array(n);
  const z = new Float32Array(n);
  const sx = new Float32Array(n);
  const sy = new Float32Array(n);
  const inFront = new Uint8Array(n);
  const tmp = {};
  for (let local = 0; local < n; local++) {
    const global = renderSelection.vertexIds[local];
    projectWorldPoint(projectionContext, mesh.vertX[global], mesh.vertY[global], mesh.vertZ[global], global, tmp);
    x[local] = tmp.x;
    y[local] = tmp.y;
    z[local] = tmp.z;
    sx[local] = tmp.sx;
    sy[local] = tmp.sy;
    inFront[local] = tmp.inFront ? 1 : 0;
  }
  return { x, y, z, sx, sy, inFront, localToGlobal: renderSelection.vertexIds };
}

function applyProjectionWobble(p, i, sx, sy, ampMul) {
  if (p.projectionWobble <= 0) return;
  const seed = (i + 1) * 409.17 + p.randomSeed * 23.91;
  const amp = p.projectionWobble * ampMul;
  sx[i] += noise(seed, 1) * amp;
  sy[i] += noise(seed, 2) * amp;
}

function buildFaces(p, verts, renderSelection) {
  const faceCount = mesh.faceCount;
  const ids = [];
  const aList = [];
  const bList = [];
  const cList = [];
  const nx = new Float32Array(faceCount);
  const ny = new Float32Array(faceCount);
  const nz = new Float32Array(faceCount);
  const tone = new Float32Array(faceCount);
  const ndotl = new Float32Array(faceCount);
  const front = new Uint8Array(faceCount);
  const inFront = new Uint8Array(faceCount);
  const offscreen = new Uint8Array(faceCount);
  const area = [];
  const cx = [];
  const cy = [];
  const depth = [];
  const minX = [];
  const minY = [];
  const maxX = [];
  const maxY = [];
  const L = lightVector(p);

  const faceIds = renderSelection?.faceIds || null;
  const loopCount = faceIds ? faceIds.length : faceCount;
  for (let faceIndex = 0; faceIndex < loopCount; faceIndex++) {
    const id = faceIds ? faceIds[faceIndex] : faceIndex;
    const aiGlobal = mesh.faceA[id], biGlobal = mesh.faceB[id], ciGlobal = mesh.faceC[id];
    const ai = renderSelection?.globalToLocalVertex ? renderSelection.globalToLocalVertex[aiGlobal] : aiGlobal;
    const bi = renderSelection?.globalToLocalVertex ? renderSelection.globalToLocalVertex[biGlobal] : biGlobal;
    const ci = renderSelection?.globalToLocalVertex ? renderSelection.globalToLocalVertex[ciGlobal] : ciGlobal;
    if (ai < 0 || bi < 0 || ci < 0) continue;
    const ax = verts.x[ai], ay = verts.y[ai], az = verts.z[ai];
    const bx = verts.x[bi], by = verts.y[bi], bz = verts.z[bi];
    const cx3 = verts.x[ci], cy3 = verts.y[ci], cz = verts.z[ci];
    const sx0 = verts.sx[ai], sy0 = verts.sy[ai];
    const sx1 = verts.sx[bi], sy1 = verts.sy[bi];
    const sx2 = verts.sx[ci], sy2 = verts.sy[ci];
    const fMinX = Math.min(sx0, sx1, sx2);
    const fMinY = Math.min(sy0, sy1, sy2);
    const fMaxX = Math.max(sx0, sx1, sx2);
    const fMaxY = Math.max(sy0, sy1, sy2);
    const isOffscreen = bboxOffscreen(p, fMinX, fMinY, fMaxX, fMaxY, p.cullMargin);
    offscreen[id] = isOffscreen ? 1 : 0;

    const abx = bx - ax, aby = by - ay, abz = bz - az;
    const acx = cx3 - ax, acy = cy3 - ay, acz = cz - az;
    const rawNx = aby * acz - abz * acy;
    const rawNy = abz * acx - abx * acz;
    const rawNz = abx * acy - aby * acx;
    const invN = 1 / (Math.hypot(rawNx, rawNy, rawNz) || 1);
    const nnx = rawNx * invN, nny = rawNy * invN, nnz = rawNz * invN;
    nx[id] = nnx; ny[id] = nny; nz[id] = nnz;
    front[id] = p.controlMode === 'freelook' ? (nnz < 0 ? 1 : 0) : (nnz > 0 ? 1 : 0);
    inFront[id] = verts.inFront[ai] && verts.inFront[bi] && verts.inFront[ci] ? 1 : 0;

    if (!isOffscreen) {
      const lit = nnx * L.x + nny * L.y + nnz * L.z;
      ndotl[id] = lit;
      const centerY = (sy0 + sy1 + sy2) / 3;
      const shade = 1 - clamp01(lit * 0.5 + 0.5);
      const rim = 1 - Math.abs(nnz);
      const contact = contactScore(p, centerY, nnz);
      let t = clamp01(shade * 0.86 + rim * p.edgeDark * 0.36 + contact * p.contact * 0.42);
      t = Math.pow(t, lerp(1.55, 0.58, clamp01(p.core / 2)));
      if (p.simplify > 0.01) {
        const bands = Math.round(lerp(10, 3, p.simplify));
        t = Math.round(t * bands) / bands;
      }
      tone[id] = t;
    }

    const fArea = Math.abs((sx1 - sx0) * (sy2 - sy0) - (sy1 - sy0) * (sx2 - sx0)) * 0.5;
    const tooSmall = p.vectorBudgetEnabled && fArea < (Number(p.vectorMinFaceAreaPx) || 0);
    if (!isOffscreen && !tooSmall && fArea > Math.max(EPS, p.cleanupMinFaceAreaPx || 0) && (p.controlMode !== 'freelook' || inFront[id])) {
      ids.push(id); aList.push(ai); bList.push(bi); cList.push(ci);
      area.push(fArea);
      cx.push((sx0 + sx1 + sx2) / 3);
      cy.push((sy0 + sy1 + sy2) / 3);
      depth.push((az + bz + cz) / 3);
      minX.push(fMinX); minY.push(fMinY); maxX.push(fMaxX); maxY.push(fMaxY);
    }
  }

  return {
    verts,
    count: ids.length,
    ids: Int32Array.from(ids),
    a: Int32Array.from(aList),
    b: Int32Array.from(bList),
    c: Int32Array.from(cList),
    area: Float32Array.from(area),
    cx: Float32Array.from(cx),
    cy: Float32Array.from(cy),
    depth: Float32Array.from(depth),
    minX: Float32Array.from(minX),
    minY: Float32Array.from(minY),
    maxX: Float32Array.from(maxX),
    maxY: Float32Array.from(maxY),
    nx, ny, nz, tone, ndotl, front, inFront, offscreen,
    detailTier: renderSelection?.faceTier || new Int8Array(faceCount),
    faceUnit: renderSelection?.faceUnit || new Int32Array(faceCount),
    renderSelection,
    visibility: new Float32Array(ids.length),
    visible: new Uint8Array(ids.length),
    flowX: new Float32Array(ids.length),
    flowY: new Float32Array(ids.length),
    visibleCount: 0,
  };
}

function buildDepthBuffer(p, verts, faceData) {
  const quality = Math.max(1, Math.ceil(Math.max(p.width, p.height) / 620));
  const w = Math.max(2, Math.floor(p.width / quality));
  const h = Math.max(2, Math.floor(p.height / quality));
  const sx = w / p.width;
  const sy = h / p.height;
  const nearIsSmaller = p.controlMode === 'freelook';
  const depth = new Float32Array(w * h);
  const owner = new Int32Array(w * h);
  depth.fill(nearIsSmaller ? 1e9 : -1e9);
  owner.fill(-1);
  let rasterized = 0;
  for (let i = 0; i < faceData.count; i++) {
    const faceId = faceData.ids[i];
    if (p.backface && !faceData.front[faceId]) continue;
    if (nearIsSmaller && !faceData.inFront[faceId]) continue;
    if (bboxOffscreen(p, faceData.minX[i], faceData.minY[i], faceData.maxX[i], faceData.maxY[i], 0)) continue;
    rasterTri(verts, faceData, i, depth, owner, w, h, sx, sy, nearIsSmaller);
    rasterized++;
  }
  return { w, h, depth, owner, sx, sy, quality, nearIsSmaller, rasterized };
}

function rasterTri(verts, faceData, i, depth, owner, w, h, sx, sy, nearIsSmaller) {
  const ai = faceData.a[i], bi = faceData.b[i], ci = faceData.c[i];
  const ax = verts.sx[ai] * sx, ay = verts.sy[ai] * sy, az = verts.z[ai];
  const bx = verts.sx[bi] * sx, by = verts.sy[bi] * sy, bz = verts.z[bi];
  const cx = verts.sx[ci] * sx, cy = verts.sy[ci] * sy, cz = verts.z[ci];
  if (nearIsSmaller && az < 0.1 && bz < 0.1 && cz < 0.1) return;
  const minX = clamp(Math.floor(faceData.minX[i] * sx) - 1, 0, w - 1);
  const maxX = clamp(Math.ceil(faceData.maxX[i] * sx) + 1, 0, w - 1);
  const minY = clamp(Math.floor(faceData.minY[i] * sy) - 1, 0, h - 1);
  const maxY = clamp(Math.ceil(faceData.maxY[i] * sy) + 1, 0, h - 1);
  const den = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy);
  if (Math.abs(den) < EPS) return;
  const faceId = faceData.ids[i];
  for (let y = minY; y <= maxY; y++) for (let x = minX; x <= maxX; x++) {
    const px = x + 0.5, py = y + 0.5;
    const u = ((by - cy) * (px - cx) + (cx - bx) * (py - cy)) / den;
    const v = ((cy - ay) * (px - cx) + (ax - cx) * (py - cy)) / den;
    const ww = 1 - u - v;
    if (u < -0.005 || v < -0.005 || ww < -0.005) continue;
    const z = u * az + v * bz + ww * cz;
    if (nearIsSmaller && z < 0.1) continue;
    const idx = y * w + x;
    if (nearIsSmaller ? z < depth[idx] : z > depth[idx]) {
      depth[idx] = z;
      owner[idx] = faceId;
    }
  }
}

function computeVisibilityAndFlow(p, faceData, db) {
  const needsFlow = p.shadowsEnabled || p.flow || p.paintEnabled || p.faceWash || p.tone;
  let visibleCount = 0;
  for (let i = 0; i < faceData.count; i++) {
    const faceId = faceData.ids[i];
    if (p.backface && !faceData.front[faceId]) continue;
    if (db.nearIsSmaller && !faceData.inFront[faceId]) continue;
    const ai = faceData.a[i], bi = faceData.b[i], ci = faceData.c[i];
    const samples = [
      [faceData.cx[i], faceData.cy[i], faceData.depth[i]],
      mixVertex(faceData.verts, ai, bi, ci, 0.60, 0.20, 0.20),
      mixVertex(faceData.verts, ai, bi, ci, 0.20, 0.60, 0.20),
      mixVertex(faceData.verts, ai, bi, ci, 0.20, 0.20, 0.60),
    ];
    let ok = 0;
    for (const s of samples) if (isVisiblePoint(p, db, s[0], s[1], s[2])) ok++;
    faceData.visibility[i] = ok / samples.length;
    faceData.visible[i] = !p.hideOccluded || ok > 0 ? 1 : 0;
    if (faceData.visible[i]) {
        if (needsFlow) computeFlow(p, faceData, i);
      visibleCount++;
    }
  }
  faceData.visibleCount = visibleCount;
}

function computeFlow(p, faceData, i) {
  const verts = faceData.verts;
  const ai = faceData.a[i], bi = faceData.b[i], ci = faceData.c[i];
  const ids = [ai, bi, ci];
  let bestX = verts.sx[bi] - verts.sx[ai];
  let bestY = verts.sy[bi] - verts.sy[ai];
  let bestLen = bestX * bestX + bestY * bestY;
  for (let k = 0; k < 3; k++) {
    const a = ids[k], b = ids[(k + 1) % 3];
    const ex = verts.sx[b] - verts.sx[a];
    const ey = verts.sy[b] - verts.sy[a];
    const len = ex * ex + ey * ey;
    if (len > bestLen) { bestX = ex; bestY = ey; bestLen = len; }
  }
  const form = norm2(bestX, bestY);
  const radial = norm2(faceData.cx[i] - p.width / 2, faceData.cy[i] - p.height / 2);
  const crossX = -radial.y, crossY = radial.x;
  const L = lightVector(p);
  const light = norm2(L.x, -L.y);
  const termX = -light.y, termY = light.x;
  const parallelAngle = deg(-22);
  const parallelX = Math.cos(parallelAngle), parallelY = Math.sin(parallelAngle);
  let outX = form.x, outY = form.y;
  switch (p.flowMode) {
    case 'parallel': outX = parallelX; outY = parallelY; break;
    case 'crossContour': ({ x: outX, y: outY } = norm2(crossX * 0.82 + form.x * 0.18, crossY * 0.82 + form.y * 0.18)); break;
    case 'silhouette': outX = crossX; outY = crossY; break;
    case 'light': outX = light.x; outY = light.y; break;
    case 'terminator': outX = termX; outY = termY; break;
    case 'form': break;
    default: ({ x: outX, y: outY } = norm2(form.x * 0.50 + crossX * 0.32 + termX * 0.20, form.y * 0.50 + crossY * 0.32 + termY * 0.20)); break;
  }
  faceData.flowX[i] = outX;
  faceData.flowY[i] = outY;
}

function computeContours(p, verts, faceData, db) {
  const out = {
    x1: [], y1: [], z1: [], x2: [], y2: [], z2: [],
    kind: [], visible: [], detailTier: [],
    tested: 0,
    count: 0,
  };
  const edgeIds = faceData.renderSelection?.edgeIds || null;
  const edgeCount = edgeIds ? edgeIds.length : mesh.edgeCount;
  const maxContours = p.vectorBudgetEnabled ? Math.max(0, Number(p.vectorMaxContourLines) || 0) : Infinity;
  for (let edgeIndex = 0; edgeIndex < edgeCount; edgeIndex++) {
    const i = edgeIds ? edgeIds[edgeIndex] : edgeIndex;
    const f0 = mesh.edgeF0[i];
    const f1 = mesh.edgeF1[i];
    const f0Selected = f0 >= 0 && (faceData.detailTier[f0] ?? 4) < 4;
    const f1Selected = f1 >= 0 && (faceData.detailTier[f1] ?? 4) < 4;
    if (!f0Selected && !f1Selected) continue;
    const primary = f0Selected ? f0 : f1;
    const secondary = f0Selected && f1Selected ? f1 : -1;
    if (faceData.offscreen[primary] && (secondary < 0 || faceData.offscreen[secondary])) continue;
    const a = mesh.edgeA[i], b = mesh.edgeB[i];
    const la = faceData.renderSelection?.globalToLocalVertex ? faceData.renderSelection.globalToLocalVertex[a] : a;
    const lb = faceData.renderSelection?.globalToLocalVertex ? faceData.renderSelection.globalToLocalVertex[b] : b;
    if (la < 0 || lb < 0) continue;
    if (bboxOffscreen(p, Math.min(verts.sx[la], verts.sx[lb]), Math.min(verts.sy[la], verts.sy[lb]), Math.max(verts.sx[la], verts.sx[lb]), Math.max(verts.sy[la], verts.sy[lb]), p.cullMargin)) continue;
    out.tested++;
    const screenLen = Math.hypot(verts.sx[la] - verts.sx[lb], verts.sy[la] - verts.sy[lb]);
    if (screenLen < (p.cleanupMinLineLengthPx || 0)) continue;
    if (p.vectorBudgetEnabled && screenLen < (Number(p.vectorMinEdgeLengthPx) || 0)) continue;
    if (screenLen > (p.cleanupMaxEdgeLengthPx || Infinity)) continue;
    const boundary = secondary < 0;
    const silhouette = secondary >= 0 ? faceData.front[primary] !== faceData.front[secondary] : true;
    const crease = secondary >= 0 ? faceDot(faceData, primary, secondary) < 0.70 : false;
    const toneBreak = secondary >= 0 ? Math.abs(faceData.tone[primary] - faceData.tone[secondary]) > 0.32 : false;
    let kind = 0;
    if (boundary || silhouette) kind = 1;
    else if (p.creases && crease) kind = 2;
    else if (p.suggestive && toneBreak) kind = 3;
    if (!kind) continue;
    const tier = faceData.renderSelection?.edgeTier?.[i] ?? Math.min(faceData.detailTier[primary] || 0, secondary >= 0 ? faceData.detailTier[secondary] || 0 : 0);
    if (!detailAllowsInternalLine(tier, kind === 1 ? 'contour' : kind === 2 ? 'crease' : 'suggestive')) continue;
    const mx = (verts.sx[la] + verts.sx[lb]) / 2;
    const my = (verts.sy[la] + verts.sy[lb]) / 2;
    const mz = (verts.z[la] + verts.z[lb]) / 2;
    const visible = isVisiblePoint(p, db, mx, my, mz);
    if (!visible && !p.showHidden) continue;
    if (out.kind.length >= maxContours) break;
    out.x1.push(verts.sx[la]); out.y1.push(verts.sy[la]); out.z1.push(verts.z[la]);
    out.x2.push(verts.sx[lb]); out.y2.push(verts.sy[lb]); out.z2.push(verts.z[lb]);
    out.kind.push(kind); out.visible.push(visible ? 1 : 0);
    out.detailTier.push(tier);
  }
  out.count = out.kind.length;
  return {
    x1: Float32Array.from(out.x1),
    y1: Float32Array.from(out.y1),
    z1: Float32Array.from(out.z1),
    x2: Float32Array.from(out.x2),
    y2: Float32Array.from(out.y2),
    z2: Float32Array.from(out.z2),
    kind: Uint8Array.from(out.kind),
    visible: Uint8Array.from(out.visible),
    detailTier: Int8Array.from(out.detailTier),
    tested: out.tested,
    count: out.count,
  };
}

function packScreenFaces(faceData) {
  const nx = new Float32Array(faceData.count);
  const ny = new Float32Array(faceData.count);
  const nz = new Float32Array(faceData.count);
  const tone = new Float32Array(faceData.count);
  const ndotl = new Float32Array(faceData.count);
  const front = new Uint8Array(faceData.count);
  const inFront = new Uint8Array(faceData.count);
  const detailTier = new Int8Array(faceData.count);
  const unitId = new Int32Array(faceData.count);
  for (let i = 0; i < faceData.count; i++) {
    const id = faceData.ids[i];
    nx[i] = faceData.nx[id];
    ny[i] = faceData.ny[id];
    nz[i] = faceData.nz[id];
    tone[i] = faceData.tone[id];
    ndotl[i] = faceData.ndotl[id];
    front[i] = faceData.front[id];
    inFront[i] = faceData.inFront[id];
    detailTier[i] = faceData.detailTier?.[id] ?? 0;
    unitId[i] = faceData.faceUnit?.[id] ?? -1;
  }
  return {
    ids: faceData.ids,
    a: faceData.a,
    b: faceData.b,
    c: faceData.c,
    nx,
    ny,
    nz,
    tone,
    ndotl,
    front,
    inFront,
    detailTier,
    unitId,
    area: faceData.area,
    cx: faceData.cx,
    cy: faceData.cy,
    depth: faceData.depth,
    minX: faceData.minX,
    minY: faceData.minY,
    maxX: faceData.maxX,
    maxY: faceData.maxY,
    visibility: faceData.visibility,
    visible: faceData.visible,
    flowX: faceData.flowX,
    flowY: faceData.flowY,
  };
}

function emptyContours() {
  return {
    x1: new Float32Array(0), y1: new Float32Array(0), z1: new Float32Array(0),
    x2: new Float32Array(0), y2: new Float32Array(0), z2: new Float32Array(0),
    kind: new Uint8Array(0), visible: new Uint8Array(0), detailTier: new Int8Array(0),
    tested: 0, count: 0,
  };
}

function lightVector(p) {
  const az = deg(p.lightAz), el = deg(p.lightEl);
  return norm3(Math.sin(az) * Math.cos(el), Math.sin(el), Math.cos(az) * Math.cos(el));
}

function bboxOffscreen(p, minX, minY, maxX, maxY, margin = 0) {
  return maxX < -margin || maxY < -margin || minX > p.width + margin || minY > p.height + margin;
}

function contactScore(p, y, nz) {
  const low = clamp01((y - p.height * 0.50) / (p.height * 0.34));
  return low * clamp01(1 - Math.abs(nz));
}

function sampleDepth(db, p, x, y) {
  if (!db || x < 0 || y < 0 || x >= p.width || y >= p.height) return db?.nearIsSmaller ? 1e9 : -1e9;
  const ix = clamp(Math.floor(x * db.sx), 0, db.w - 1);
  const iy = clamp(Math.floor(y * db.sy), 0, db.h - 1);
  return db.depth[iy * db.w + ix];
}

function isVisiblePoint(p, db, x, y, z) {
  if (!p.hideOccluded || !p.depthClipStrokes) return x >= 0 && y >= 0 && x < p.width && y < p.height;
  if (db?.nearIsSmaller) {
    if (z < 0.1) return false;
    return z <= sampleDepth(db, p, x, y) + p.depthEps;
  }
  return z >= sampleDepth(db, p, x, y) - p.depthEps;
}

function mixVertex(verts, a, b, c, u, v, w) {
  return [
    verts.sx[a] * u + verts.sx[b] * v + verts.sx[c] * w,
    verts.sy[a] * u + verts.sy[b] * v + verts.sy[c] * w,
    verts.z[a] * u + verts.z[b] * v + verts.z[c] * w,
  ];
}

function faceDot(faceData, a, b) {
  return faceData.nx[a] * faceData.nx[b] + faceData.ny[a] * faceData.ny[b] + faceData.nz[a] * faceData.nz[b];
}

function norm3(x, y, z) {
  const len = Math.hypot(x, y, z) || 1;
  return { x: x / len, y: y / len, z: z / len };
}

function norm2(x, y) {
  const len = Math.hypot(x, y) || 1;
  return { x: x / len, y: y / len };
}
