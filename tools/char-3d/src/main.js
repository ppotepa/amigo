import { state } from './state/defaultState.js';
import { rangeControls, colorControls, checkControls } from './state/controlSchema.js';
import { presets } from './state/stylePresets.js';
import { paintPalettes } from './state/paintPalettes.js';
import { parseOBJ } from './mesh/objParser.js';
import { prepareMeshRuntime } from './mesh/meshRuntime.js';
import { extractFbxAdapterMesh, ensureFbxRuntime as ensureFbxRuntimeForState, FBX_MODEL_URL } from './mesh/fbxAdapter.js';
import { buildFbxClipAmc, downloadArrayBuffer } from './mesh/fbxClipBake.js';
import { BUILTIN_MODELS } from './mesh/modelSources.js';
import { TAU, EPS, clamp, clamp01, lerp, deg, fmt, v3, sub, cross, dot, norm, len2, norm2, rot2, mix2, triArea2, hash01, noise, bary2, baryInside, mixPoint, pointFromBary } from './math/core.js';
import { escapeHtml, normalizeHexColor, hexRgb, mixRgb, rgba } from './math/color.js';
import { cameraKeyCodes, isTypingTarget, setModelAngles as setModelAnglesForState, setCameraAngles as setCameraAnglesForState, applyAngleSnap as applyAngleSnapForState, cameraDollyScale as cameraDollyScaleForState, updateCameraFromKeys as updateCameraFromKeysForState } from './app/cameraControls.js';
import { computeImpreciseSampleTime as computeImpreciseSampleTimeForState, randomnessFrameSeed as randomnessFrameSeedForState, shadowRandomSeed as shadowRandomSeedForState } from './npr/randomSeeds.js';
import { buildPaintRegions } from './paint/paintRegions.js';
import { buildScenePartition, getScenePartitionKey } from './scene/scenePartition.js';
import { createProjectionContext, projectWorldPoint } from './render/projectionContext.js';
import { selectVisibleRenderUnits } from './render/visibilitySelection.js';
import { assignDetailTiers, detailAllowsInternalLine, detailMarkMultiplier } from './render/detailPolicy.js';
import { buildRenderSelection, buildFullRenderSelection } from './render/renderSelection.js';
import { createPerfStats, resetPerfFrame, markCacheHit, markCacheMiss, timeSection, timeSectionEnd, finishPerfFrame, setPerfCounter, formatPerfStats } from './render/perfStats.js';
import {
  createRenderCache,
  buildPipelineKey,
  buildBackgroundLayerKey,
  buildPaintLayerKey,
  buildSvgKey,
  ensureLayerCanvas,
  clearFrameLists,
  getReusableDepthBuffers,
  invalidateDerivedCaches
} from './render/renderCache.js';
import {
  buildFrameWorkerMeshPayload,
  meshFrameWorkerKey,
  shouldUseFrameWorker,
  snapshotFrameWorkerParams
} from './render/frameProtocol.js';
import { DIRTY_FLAGS, createDirtyFlags, markDirty, clearDirty } from './render/dirtyFlags.js';

'use strict';

  const canvas = document.getElementById('view');
  const ctx = canvas.getContext('2d', { alpha: false });
  const statusEl = document.getElementById('status');
  const legendEl = document.getElementById('legend');

  const $ = id => document.getElementById(id);

  const STORAGE_KEY = 'char3d.strokes.settings.v7';
  const persistedFields = [
    ...Object.keys(rangeControls),
    ...colorControls,
    ...checkControls,
    'mainContourTool',
    'creaseAccentTool',
    'suggestiveContourTool',
    'hiddenLineTool',
    'shadowHatchTool',
    'controlMode',
    'angleSnap',
    'projectionMode',
    'method',
    'flowMode',
    'preset',
    'paintBrush',
    'paintPalette',
    'modelSource',
    'animFps',
    'mode',
    'cameraYaw',
    'cameraPitch',
    'cameraX',
    'cameraY',
    'cameraZ',
    'focalLength',
    'rawYaw',
    'rawPitch',
    'rawCameraYaw',
    'rawCameraPitch'
  ];
  const lineToolControls = ['mainContourTool','creaseAccentTool','suggestiveContourTool','hiddenLineTool','shadowHatchTool'];
  let settingsLoaded = false;
  let saveSettingsTimer = 0;
  let meshRevision = 0;
  let objLoadToken = 0;
  let objParseSeq = 0;
  let objParseWorker = null;
  let frameWorker = null;
  let frameWorkerMeshKey = '';
  let frameWorkerJobSeq = 0;
  let frameWorkerPending = null;
  let frameWorkerDropped = 0;
  const renderCache = createRenderCache();
  const perfStats = createPerfStats();
  const dirtyFlags = createDirtyFlags();
  const builtinModelMap = new Map(BUILTIN_MODELS.map(model => [model.id, model]));
  const displayOnlyKeys = new Set(['paintEnabled','faceWash','contours','tone','flow','depthDebug','seedDebug','regionDebug','cleanupDebug','densityDebug','visibilityDebug','detailDebug','budgetDebug','sortFaces','inkDominance']);
  const visibilityKeys = new Set(['hideOccluded','backface','depthClipStrokes','clipToFaces','showHidden','depthEps','creases','suggestive','contactLines']);
  const selectionKeys = new Set([
    'scenePartitionEnabled','scenePartitionMode','scenePartitionCellSize','scenePartitionMaxUnits',
    'visibilityCullingEnabled','visibilityMarginPx','visibilityMinAreaPx','visibilityMinRadiusPx',
    'detailPolicyEnabled','detailTier0RadiusPx','detailTier1RadiusPx','detailTier2RadiusPx','detailTier3RadiusPx',
    'detailDensityPenalty','detailImportanceBias',
    'vectorBudgetEnabled','vectorMaxProjectedFaces','vectorMaxVisibleEdges','vectorMaxContourLines',
    'vectorMinFaceAreaPx','vectorMinEdgeLengthPx'
  ]);
  const budgetDisplayKeys = new Set(['visibilityDebug','detailDebug','densityDebug','budgetDebug','regionDebug','cleanupDebug','depthDebug','seedDebug']);
  const nprKeys = new Set(['method','mode','flowMode','density','layers','threshold','strokeLen','spacing','strokeWidth','curvature','crossAngle','dotSize','wobble','jitter','strokeCrookedness','strokeKinkChance','strokeToneRamp','shadowFrameDrift','shadowLoopRedraw','shadowLayoutJitter','spacingVar','lengthVar','widthVar','taper','breakup','overdraw','contourHumanize','contourDrift','contourWobble','contourGaps','contourFrameVariance','shadowsEnabled','vectorMaxShadowMarks','mainContourEnabled','creaseAccentEnabled','suggestiveContourEnabled','hiddenLineEnabled','shadowHatchEnabled','mainContourTool','creaseAccentTool','suggestiveContourTool','hiddenLineTool','shadowHatchTool']);

  function renderScopeForKey(key) {
    if (budgetDisplayKeys.has(key)) return DIRTY_FLAGS.DISPLAY;
    if (String(key).startsWith('paint')) return DIRTY_FLAGS.PAINT;
    if (String(key).startsWith('region')) return DIRTY_FLAGS.PAINT;
    if (String(key).startsWith('cleanupRegion') || key === 'hairRegionSuppression' || key === 'shadowBandCount' || key === 'shadowRegionBleed' || key === 'shadowColorJitter' || key === 'baseWashEnabled' || key === 'shadowRegionEnabled' || key === 'highlightRegionEnabled') return DIRTY_FLAGS.PAINT;
    if (String(key).startsWith('cleanup') || key === 'temporalCoherence' || key === 'projectionHumanError') return DIRTY_FLAGS.PROJECTION;
    if (key === 'strokePressureJitter') return DIRTY_FLAGS.NPR;
    if (selectionKeys.has(key)) return DIRTY_FLAGS.PROJECTION;
    if (displayOnlyKeys.has(key)) return DIRTY_FLAGS.DISPLAY;
    if (visibilityKeys.has(key)) return DIRTY_FLAGS.VISIBILITY;
    if (nprKeys.has(key)) return DIRTY_FLAGS.NPR;
    return DIRTY_FLAGS.PROJECTION;
  }

  function paperColor() { return normalizeHexColor(state.paintPaperColor, '#f4eee3'); }

  function resizeCanvas() {
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    const w = Math.max(320, Math.floor(window.innerWidth * dpr));
    const h = Math.max(240, Math.floor(window.innerHeight * dpr));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
      invalidateDerivedCaches(renderCache);
    }
  }

  function lightVector() {
    const az = deg(state.lightAz), el = deg(state.lightEl);
    return norm(v3(Math.sin(az)*Math.cos(el), Math.sin(el), Math.cos(az)*Math.cos(el)));
  }

  function wrapTime(value, duration) {
    if (!duration || duration <= 0) return value;
    let out = value % duration;
    if (out < 0) out += duration;
    return out;
  }

  function computeImpreciseSampleTime(step, duration) {
    return computeImpreciseSampleTimeForState(state, step, duration);
  }

  function randomnessFrameSeed() {
    return randomnessFrameSeedForState(state);
  }

  function shadowRandomSeed() {
    return shadowRandomSeedForState(state);
  }

  function setCameraAngles(yaw, pitch) {
    setCameraAnglesForState(state, yaw, pitch);
  }

  function setModelAngles(yaw, pitch) {
    setModelAnglesForState(state, yaw, pitch);
  }

  function applyAngleSnap() {
    applyAngleSnapForState(state);
  }

  function cameraDollyScale() {
    return cameraDollyScaleForState(state);
  }

  function cullMargin() {
    return Math.max(32, state.strokeLen * 1.5 + state.spacing * state.jitter * 2 + state.paintBleed * 16);
  }

  function bboxOffscreen(minX, minY, maxX, maxY, margin = 0) {
    return maxX < -margin || maxY < -margin || minX > canvas.width + margin || minY > canvas.height + margin;
  }

  function writeProjectedVertex(out, x, y, z, sx, sy, inFront) {
    out.x = x;
    out.y = y;
    out.z = z;
    out.sx = sx;
    out.sy = sy;
    out.inFront = inFront;
    return out;
  }

  function writeNorm2(out, x, y) {
    const length = Math.hypot(x, y) || 1;
    out.x = x / length;
    out.y = y / length;
    return out;
  }

  function createFrameProjectionContext(runtime) {
    const mesh = state.mesh;
    const fbxAdapter = mesh?.sourceType === 'fbx';
    const freelook = state.controlMode === 'freelook';
    const centerX = canvas.width / 2 + (!freelook && fbxAdapter ? Math.min(180, canvas.width * .13) : 0);
    const centerY = canvas.height / 2 - (!freelook && fbxAdapter ? Math.min(42, canvas.height * .04) : 0);
    return createProjectionContext({
      width: canvas.width,
      height: canvas.height,
      controlMode: state.controlMode,
      projectionMode: state.projectionMode,
      yaw: state.yaw,
      pitch: state.pitch,
      zoom: state.zoom,
      cameraYaw: state.cameraYaw,
      cameraPitch: state.cameraPitch,
      cameraX: state.cameraX,
      cameraY: state.cameraY,
      cameraZ: state.cameraZ,
      focalLength: state.focalLength,
      projectionWobble: state.projectionWobble,
      randomSeed: randomnessFrameSeed(),
      cameraDollyScale: cameraDollyScale(),
      centerX,
      centerY,
      sourceScaleMul: fbxAdapter ? .78 : 1,
      sourceWobbleMul: fbxAdapter ? 1.18 : 1,
      runtime,
    });
  }

  function getOrBuildScenePartition(runtime) {
    if (!runtime) return null;
    const key = getScenePartitionKey(runtime, state);
    if (renderCache.partition?.key === key && renderCache.partition.value) return renderCache.partition.value;
    const partition = buildScenePartition(runtime, state);
    renderCache.partition = { key, value: partition };
    return partition;
  }

  function buildFrameRenderSelection(runtime, projectionContext) {
    if (!runtime) return null;
    if (!state.scenePartitionEnabled && !state.visibilityCullingEnabled && !state.detailPolicyEnabled && !state.vectorBudgetEnabled) {
      return buildFullRenderSelection(runtime);
    }
    const partition = getOrBuildScenePartition(runtime);
    const viewport = { width: canvas.width, height: canvas.height };
    const visibility = timeSection(perfStats, 'selection', () => selectVisibleRenderUnits(partition, projectionContext, viewport, state));
    const detailed = assignDetailTiers(visibility, state);
    const selection = buildRenderSelection(detailed, runtime, state);
    for (const [name, value] of Object.entries(visibility.counters || {})) setPerfCounter(perfStats, name, value);
    for (const [name, value] of Object.entries(detailed.counters || {})) setPerfCounter(perfStats, name, value);
    for (const [name, value] of Object.entries(selection.counters || {})) setPerfCounter(perfStats, name, value);
    return selection;
  }

  function transformFrame(renderSelection, projectionContext) {
    const mesh = state.mesh;
    const runtime = prepareMeshRuntime(mesh);
    const lists = clearFrameLists(renderCache);
    const freelook = state.controlMode === 'freelook';
    const margin = cullMargin();
    const verts = lists.verts;
    const vertX = runtime.vertX;
    const vertY = runtime.vertY;
    const vertZ = runtime.vertZ;
    renderSelection ||= buildFullRenderSelection(runtime);
    projectionContext ||= createFrameProjectionContext(runtime);

    for (let i = 0; i < renderSelection.vertexIds.length; i++) {
      const vi = renderSelection.vertexIds[i];
      const out = verts[vi] || {};
      projectWorldPoint(projectionContext, vertX[vi], vertY[vi], vertZ[vi], vi, out);
      verts[vi] = out;
    }
    verts.length = runtime.vertCount;

    const L = lightVector();
    const faces = lists.faces;
    for (let ii = 0; ii < renderSelection.faceIds.length; ii++) {
      const id = renderSelection.faceIds[ii];
      const a=verts[runtime.faceA[id]], b=verts[runtime.faceB[id]], c=verts[runtime.faceC[id]];
      if (!a || !b || !c) continue;
      const minX = Math.min(a.sx, b.sx, c.sx);
      const minY = Math.min(a.sy, b.sy, c.sy);
      const maxX = Math.max(a.sx, b.sx, c.sx);
      const maxY = Math.max(a.sy, b.sy, c.sy);
      const offscreen = bboxOffscreen(minX, minY, maxX, maxY, margin);
      const out = faces[id] || {p:[null,null,null], n:{x:0,y:0,z:0}, flow:{x:1,y:0}};
      const n = out.n;
      const abx = b.x - a.x, aby = b.y - a.y, abz = b.z - a.z;
      const acx = c.x - a.x, acy = c.y - a.y, acz = c.z - a.z;
      const nx = aby * acz - abz * acy;
      const ny = abz * acx - abx * acz;
      const nz = abx * acy - aby * acx;
      const invN = 1 / (Math.hypot(nx, ny, nz) || 1);
      n.x = nx * invN;
      n.y = ny * invN;
      n.z = nz * invN;
      const cx=(a.sx+b.sx+c.sx)/3, cy2=(a.sy+b.sy+c.sy)/3;
      const depth=(a.z+b.z+c.z)/3;
      const area=triArea2(a,b,c);
      const ndotl=offscreen ? 0 : dot(n,L);
      let tone = 0;
      if (!offscreen) {
        const shade = 1 - clamp01(ndotl * .5 + .5);
        const rim = 1 - Math.abs(n.z);
        const contact = contactScore(cy2, n);
        tone = clamp01(shade * .86 + rim * state.edgeDark * .36 + contact * state.contact * .42);
        tone = Math.pow(tone, lerp(1.55, .58, clamp01(state.core/2)));
        if (state.simplify > 0.01) {
          const bands = Math.round(lerp(10, 3, state.simplify));
          tone = Math.round(tone * bands) / bands;
        }
      }
      const front = freelook ? n.z < 0 : n.z > 0;
      const inFront = Boolean(a.inFront && b.inFront && c.inFront);
      const tooSmall = state.vectorBudgetEnabled && Math.abs(area) < (Number(state.vectorMinFaceAreaPx) || 0);
      out.id = id;
      out.p[0] = a;
      out.p[1] = b;
      out.p[2] = c;
      out.area = area;
      out.cx = cx;
      out.cy = cy2;
      out.depth = depth;
      out.front = front;
      out.inFront = inFront;
      out.tone = tone;
      out.ndotl = ndotl;
      out.flow.x = 1;
      out.flow.y = 0;
      out.visible = false;
      out.visibility = 0;
      out.minX = minX;
      out.minY = minY;
      out.maxX = maxX;
      out.maxY = maxY;
      out.offscreen = offscreen;
      out.tooSmall = tooSmall;
      out.detailTier = renderSelection.faceTier?.[id] ?? 0;
      out.unitId = renderSelection.faceUnit?.[id] ?? -1;
      faces[id] = out;
      if (!offscreen && !tooSmall && Math.abs(area) > EPS && (!freelook || inFront)) lists.screenFaces.push(out);
    }
    faces.length = runtime.faceCount;
    setPerfCounter(perfStats, 'facesTotal', runtime.faceCount);
    setPerfCounter(perfStats, 'facesSelected', renderSelection.faceIds.length);
    setPerfCounter(perfStats, 'vertsSelected', renderSelection.vertexIds.length);
    setPerfCounter(perfStats, 'edgesSelected', renderSelection.edgeIds.length);
    setPerfCounter(perfStats, 'facesOnScreen', lists.screenFaces.length);
    return {verts, faces, screenFaces:lists.screenFaces, visibleFaces:lists.visibleFaces, sortedFaces:lists.sortedFaces, L, db:null, contours:[], marks:[], depthMode: freelook ? 'min' : 'max', cullMargin:margin, viewport:{width:canvas.width, height:canvas.height, margin}, renderSelection};
  }

  function contactScore(y, n) {
    const low = clamp01((y - canvas.height * .50) / (canvas.height * .34));
    const grazing = clamp01(1 - Math.abs(n.z));
    return low * grazing;
  }

  function buildDepthBuffer(frame) {
    const quality = Math.max(1, Math.ceil(Math.max(canvas.width, canvas.height) / 620));
    const w = Math.max(2, Math.floor(canvas.width / quality));
    const h = Math.max(2, Math.floor(canvas.height / quality));
    const reusable = getReusableDepthBuffers(renderCache, w, h);
    const depth = reusable.depth;
    const owner = reusable.owner;
    const sx = w / canvas.width, sy = h / canvas.height;
    const nearIsSmaller = frame.depthMode === 'min';
    depth.fill(nearIsSmaller ? 1e9 : -1e9);
    let rasterized = 0;
    for (const f of frame.screenFaces) {
      if (state.backface && !f.front) continue;
      if (nearIsSmaller && !f.inFront) continue;
      if (bboxOffscreen(f.minX, f.minY, f.maxX, f.maxY, 0)) continue;
      rasterTri(f, depth, owner, w, h, sx, sy, nearIsSmaller);
      rasterized++;
    }
    setPerfCounter(perfStats, 'facesDepth', rasterized);
    return {w,h,depth,owner,sx,sy,quality,nearIsSmaller};
  }

  function rasterTri(f, depth, owner, w, h, sx, sy, nearIsSmaller) {
    const p=f.p;
    const ax=p[0].sx*sx, ay=p[0].sy*sy, az=p[0].z;
    const bx=p[1].sx*sx, by=p[1].sy*sy, bz=p[1].z;
    const cx=p[2].sx*sx, cy=p[2].sy*sy, cz=p[2].z;
    if (nearIsSmaller && az < 0.1 && bz < 0.1 && cz < 0.1) return;
    const minX=clamp(Math.floor(f.minX*sx)-1,0,w-1), maxX=clamp(Math.ceil(f.maxX*sx)+1,0,w-1);
    const minY=clamp(Math.floor(f.minY*sy)-1,0,h-1), maxY=clamp(Math.ceil(f.maxY*sy)+1,0,h-1);
    const den = (by-cy)*(ax-cx) + (cx-bx)*(ay-cy);
    if (Math.abs(den) < EPS) return;
    for (let y=minY;y<=maxY;y++) for (let x=minX;x<=maxX;x++) {
      const px=x+.5, py=y+.5;
      const u=((by-cy)*(px-cx)+(cx-bx)*(py-cy))/den;
      const v=((cy-ay)*(px-cx)+(ax-cx)*(py-cy))/den;
      const ww=1-u-v;
      if (u < -0.005 || v < -0.005 || ww < -0.005) continue;
      const z = u*az + v*bz + ww*cz;
      if (nearIsSmaller && z < 0.1) continue;
      const idx=y*w+x;
      if (nearIsSmaller ? z < depth[idx] : z > depth[idx]) { depth[idx]=z; owner[idx]=f.id; }
    }
  }

  function sampleDepth(db, x, y) {
    if (!db || x<0 || y<0 || x>=canvas.width || y>=canvas.height) return db?.nearIsSmaller ? 1e9 : -1e9;
    const ix=clamp(Math.floor(x*db.sx),0,db.w-1), iy=clamp(Math.floor(y*db.sy),0,db.h-1);
    return db.depth[iy*db.w+ix];
  }

  function isVisiblePoint(db, x, y, z) {
    if (!state.hideOccluded || !state.depthClipStrokes) return x>=0 && y>=0 && x<canvas.width && y<canvas.height;
    if (db?.nearIsSmaller) {
      if (z < 0.1) return false;
      return z <= sampleDepth(db, x, y) + state.depthEps;
    }
    return z >= sampleDepth(db, x, y) - state.depthEps;
  }

  function computeVisibilityAndFlow(frame) {
    const db = frame.db;
    const needsFlow = state.shadowsEnabled || state.flow || state.paintEnabled || state.faceWash || state.tone;
    frame.visibleFaces.length = 0;
    for (const f of frame.screenFaces) {
      const p=f.p;
      if (state.backface && !f.front) { f.visible=false; f.visibility=0; continue; }
      if (db?.nearIsSmaller && !f.inFront) { f.visible=false; f.visibility=0; continue; }
      const samples = [
        {x:f.cx,y:f.cy,z:f.depth},
        mixPoint(p[0],p[1],p[2],.60,.20,.20),
        mixPoint(p[0],p[1],p[2],.20,.60,.20),
        mixPoint(p[0],p[1],p[2],.20,.20,.60)
      ];
      let ok=0;
      for (const s of samples) if (isVisiblePoint(db,s.x,s.y,s.z)) ok++;
      f.visibility = ok / samples.length;
      f.visible = !state.hideOccluded ? true : ok > 0;
      if (f.visible) {
        if (needsFlow) computeFlow(f, frame);
        frame.visibleFaces.push(f);
      }
    }
    setPerfCounter(perfStats, 'facesVisible', frame.visibleFaces.length);
  }

  function computeFlow(f, frame) {
    const p=f.p;
    let bestX = p[1].sx-p[0].sx;
    let bestY = p[1].sy-p[0].sy;
    let bestLen = bestX * bestX + bestY * bestY;
    for (let i=0;i<3;i++) {
      const a=p[i], b=p[(i+1)%3];
      const ex = b.sx-a.sx;
      const ey = b.sy-a.sy;
      const l = ex * ex + ey * ey;
      if (l > bestLen) { bestX = ex; bestY = ey; bestLen = l; }
    }
    const form = writeNorm2(computeFlow.tmpForm, bestX, bestY);
    const radial = writeNorm2(computeFlow.tmpRadial, f.cx-canvas.width/2, f.cy-canvas.height/2);
    const crossX = -radial.y;
    const crossY = radial.x;
    const light = writeNorm2(computeFlow.tmpLight, frame.L.x, -frame.L.y);
    const termX = -light.y;
    const termY = light.x;
    const parallelAngle = deg(-22);
    const parallelX = Math.cos(parallelAngle);
    const parallelY = Math.sin(parallelAngle);
    const out = f.flow;
    switch (state.flowMode) {
      case 'parallel': out.x = parallelX; out.y = parallelY; return out;
      case 'form': out.x = form.x; out.y = form.y; return out;
      case 'crossContour': return writeNorm2(out, crossX*.82 + form.x*.18, crossY*.82 + form.y*.18);
      case 'silhouette': out.x = crossX; out.y = crossY; return out;
      case 'light': out.x = light.x; out.y = light.y; return out;
      case 'terminator': out.x = termX; out.y = termY; return out;
      default: return writeNorm2(out, form.x*.50 + crossX*.32 + termX*.20, form.y*.50 + crossY*.32 + termY*.20);
    }
  }
  computeFlow.tmpForm = {x:1,y:0};
  computeFlow.tmpRadial = {x:1,y:0};
  computeFlow.tmpLight = {x:1,y:0};

  function computeContours(frame) {
    const out=[];
    const mesh=state.mesh;
    const runtime = prepareMeshRuntime(mesh);
    const fbxAdapter = mesh?.sourceType === 'fbx';
    let tested = 0;
    const edgeIds = frame.renderSelection?.edgeIds || null;
    const edgeCount = edgeIds ? edgeIds.length : runtime.edgeCount;
    const maxContours = state.vectorBudgetEnabled ? Math.max(0, Number(state.vectorMaxContourLines) || 0) : Infinity;
    for (let edgeIndex = 0; edgeIndex < edgeCount; edgeIndex++) {
      const i = edgeIds ? edgeIds[edgeIndex] : edgeIndex;
      const f0Id = runtime.edgeF0[i];
      const f1Id = runtime.edgeF1[i];
      const selectedFaceTier = frame.renderSelection?.faceTier || null;
      let f0 = f0Id >= 0 && (!selectedFaceTier || selectedFaceTier[f0Id] < 4) ? frame.faces[f0Id] : null;
      let f1 = f1Id >= 0 && (!selectedFaceTier || selectedFaceTier[f1Id] < 4) ? frame.faces[f1Id] : null;
      if (!f0 && !f1) continue;
      if (!f0) {
        f0 = f1;
        f1 = null;
      }
      if (f0?.offscreen && (f1?.offscreen ?? true)) continue;
      const a=frame.verts[runtime.edgeA[i]], b=frame.verts[runtime.edgeB[i]];
      if (!a || !b) continue;
      const boundary=!f1;
      const silhouette=f1 ? (f0.front !== f1.front) : true;
      const crease=f1 ? dot(f0.n, f1.n) < .70 : false;
      const toneBreak=f1 ? Math.abs(f0.tone - f1.tone) > .32 : false;
      if (bboxOffscreen(Math.min(a.sx, b.sx), Math.min(a.sy, b.sy), Math.max(a.sx, b.sx), Math.max(a.sy, b.sy), frame.cullMargin)) continue;
      tested++;
      const screenLen = Math.hypot(a.sx-b.sx, a.sy-b.sy);
      if (screenLen < (state.cleanupMinLineLengthPx || 0)) continue;
      if (state.vectorBudgetEnabled && screenLen < (Number(state.vectorMinEdgeLengthPx) || 0)) continue;
      if (screenLen > (state.cleanupMaxEdgeLengthPx || Infinity)) continue;
      let kind='';
      if (boundary || silhouette) kind='contour';
      else if (!fbxAdapter && state.creases && crease) kind='crease';
      else if (!fbxAdapter && state.suggestive && toneBreak) kind='suggestive';
      if (!kind) continue;
      const tier = frame.renderSelection?.edgeTier?.[i] ?? Math.min(f0?.detailTier ?? 0, f1?.detailTier ?? 0);
      if (!detailAllowsInternalLine(tier, kind)) continue;
      if (fbxAdapter && screenLen < 2.4) continue;
      const mx=(a.sx+b.sx)/2, my=(a.sy+b.sy)/2, mz=(a.z+b.z)/2;
      const visible = isVisiblePoint(frame.db, mx, my, mz);
      if (!visible && !state.showHidden) continue;
      if (out.length >= maxContours) break;
      out.push({x1:a.sx,y1:a.sy,z1:a.z,x2:b.sx,y2:b.sy,z2:b.z,kind,visible,id:out.length,detailTier:tier});
    }
    setPerfCounter(perfStats, 'contoursTested', tested);
    setPerfCounter(perfStats, 'contoursDrawn', out.length);
    setPerfCounter(perfStats, 'contoursBudget', Number.isFinite(maxContours) ? maxContours : 0);
    setPerfCounter(perfStats, 'contoursBudgetHit', out.length >= maxContours ? 1 : 0);
    return out;
  }

  function generateMarks(frame) {
    const marks=[];
    const fbxAdapter = state.mesh?.sourceType === 'fbx';
    const legacyMinArea = fbxAdapter ? 0.22 : 1.5;
    const minArea = state.vectorBudgetEnabled
      ? Math.max(legacyMinArea, state.cleanupMinFaceAreaPx || 0, Number(state.vectorMinFaceAreaPx) || 0)
      : Math.max(legacyMinArea, state.cleanupMinFaceAreaPx || 0);
    const faces = frame.sortedFaces;
    faces.length = 0;
    for (const f of frame.visibleFaces) if ((f.detailTier ?? 0) < 4 && f.area > minArea && (!state.backface || f.front)) faces.push(f);
    faces.sort((a,b)=>{
      const ta = a.detailTier ?? 0;
      const tb = b.detailTier ?? 0;
      if (ta !== tb) return ta - tb;
      return b.depth-a.depth;
    });
    const baseMarks = fbxAdapter ? 760 : 450;
    const markRange = fbxAdapter ? 2600 : 1800;
    const densityClamp = clamp01(state.cleanupDensityClamp ?? 0.65);
    const legacyMaxMarks = Math.floor(baseMarks + markRange * clamp01(state.density/2) * densityClamp * lerp(1.15,.45,state.economy));
    const maxMarks = state.vectorBudgetEnabled
      ? Math.min(legacyMaxMarks, Math.max(0, Math.floor(state.vectorMaxShadowMarks || 0)))
      : legacyMaxMarks;
    let used=0;
    for (const f of faces) {
      if (used >= maxMarks) break;
      const made = generateFaceMarks(f, frame, marks, maxMarks-used);
      used += made;
    }
    setPerfCounter(perfStats, 'marksGenerated', marks.length);
    setPerfCounter(perfStats, 'marksBudget', maxMarks);
    setPerfCounter(perfStats, 'marksBudgetHit', used >= maxMarks ? 1 : 0);
    return marks;
  }

  function effectiveTone(f) {
    let tone = clamp01((f.tone - state.threshold) / Math.max(0.05, 1 - state.threshold));
    if (state.contactLines) tone = clamp01(tone + contactScore(f.cy, f.n) * state.contact * .72);
    if (state.method === 'comic') tone = Math.pow(tone, .72);
    return tone;
  }

  function generateFaceMarks(f, frame, marks, budget) {
    const fbxAdapter = state.mesh?.sourceType === 'fbx';
    const tone=effectiveTone(f);
    if (tone <= .018 && !['stipple','halftone'].includes(state.method)) return 0;
    const spacing = Math.max(fbxAdapter ? 2 : 3, state.spacing * lerp(1.45,.55,tone) * lerp(.58,1.85,state.economy) * (fbxAdapter ? .72 : 1));
    const layerBoost = ['crosshatch','graphite'].includes(state.method) ? Math.max(1,state.layers) : 1;
    let raw = f.area / (spacing*spacing) * state.density * lerp(.6,2.25,tone) * layerBoost * f.visibility;
    if (state.method === 'halftone') raw = f.area / (spacing*spacing) * state.density * lerp(.7,2.7,tone) * f.visibility;
    if (state.method === 'stipple') raw = f.area / (spacing*spacing) * state.density * lerp(.7,3.0,tone) * f.visibility;
    if (state.method === 'comic' && tone > .70) raw *= 1.35;
    if (fbxAdapter) raw *= 1.45;
    const tier = f.detailTier ?? 0;
    if (tier >= 4) return 0;
    raw *= detailMarkMultiplier(tier);
    const shadowSeed = shadowRandomSeed() * lerp(1, .15, clamp01(state.temporalCoherence ?? 0.85));
    const seed=(f.id+1)*1009.133 + shadowSeed * (1 + hash01(f.id + 19.3) * .7);
    let n = Math.floor(raw);
    if (hash01(seed + 991.7) < raw - n) n++;
    if (tone > .08 && raw > .045 && hash01(seed + 113.9) < raw * 1.8) n = Math.max(n, 1);
    const perFaceMax = tier <= 0 ? 42 : tier === 1 ? 24 : tier === 2 ? 8 : tier === 3 ? 2 : 0;
    n = clamp(n, 0, Math.min(perFaceMax, budget));
    let made=0;
    for (let i=0; i<n && made<budget; i++) {
      const b = stableBary(seed, i, state.spacingVar);
      const c = pointFromBary(f, b.u, b.v, b.w);
      c.x += noise(seed, i+10) * state.jitter * spacing * .35;
      c.y += noise(seed, i+20) * state.jitter * spacing * .35;
      c.x += noise(seed, i+30) * (state.projectionHumanError || 0) * spacing * .28;
      c.y += noise(seed, i+40) * (state.projectionHumanError || 0) * spacing * .28;
      const bc = bary2(c, f.p[0], f.p[1], f.p[2]);
      if (state.clipToFaces && !baryInside(bc, .035)) continue;
      if (bc) c.z = bc.u*f.p[0].z + bc.v*f.p[1].z + bc.w*f.p[2].z;
      if (!isVisiblePoint(frame.db, c.x, c.y, c.z)) continue;
      addMark(marks, f, frame, c, tone, seed + i * 73.19);
      made++;
    }
    return made;
  }

  function stableBary(seed, i, variance) {
    let r1=hash01(seed + i*2.17), r2=hash01(seed + i*5.91 + 11.3);
    if (variance < .98) {
      const grid = Math.max(2, Math.round(lerp(9,3,variance)));
      r1 = (Math.floor(r1*grid) + .5 + noise(seed,i)*variance*.45) / grid;
      r2 = (Math.floor(r2*grid) + .5 + noise(seed+5,i)*variance*.45) / grid;
      r1 = clamp01(r1); r2 = clamp01(r2);
    }
    const s=Math.sqrt(r1);
    return {u:1-s, v:s*(1-r2), w:s*r2};
  }

  function strokeTool(toolId) {
    return state.strokeTools?.[toolId] || state.strokeTools?.mainInk || {
      type: 'pen',
      color: '#17110b',
      alphaRange: [0.72, 0.96],
      widthRange: [0.8, 1.7],
      taper: 0.25,
      wobble: 0.18,
      dryness: 0.05,
    };
  }

  function colorFromTool(tool, tone, seed) {
    if (tool.colorRange?.length >= 2) {
      const a = hexRgb(tool.colorRange[0], '#2f2a25');
      const b = hexRgb(tool.colorRange[1], '#716456');
      const jitter = (state.shadowColorJitter || 0) * noise(seed || 0, 91) * .35;
      return rgba(mixRgb(a, b, clamp01(tone + jitter)), 1);
    }
    return tool.color || '#17110b';
  }

  function resolveStrokeStyle({ toolId = 'mainInk', lineSetId = '', tone = 1, seed = 0 } = {}) {
    const tool = strokeTool(toolId);
    const alphaRange = tool.alphaRange || [0.45, 0.9];
    const widthRange = tool.widthRange || [0.5, 1.2];
    let alpha = lerp(alphaRange[0], alphaRange[1], clamp01(tone));
    let width = lerp(widthRange[0], widthRange[1], clamp01(tone));

    // Legacy mode still acts as a coarse compatibility multiplier, but the tool owns identity.
    if (state.mode === 'PENCIL' && tool.type !== 'pencil') { alpha *= .62; width *= .78; }
    if (state.mode === 'BRUSH' && tool.type !== 'brush') { alpha *= .82; width *= 1.35; }

    const pressure = 1 + noise(seed, 111) * (state.strokePressureJitter || 0) * .35;
    return {
      toolId,
      lineSetId,
      color: colorFromTool(tool, tone, seed),
      alpha: clamp01(alpha * lerp(.48, 1, tone)),
      width: Math.max(.15, width * state.strokeWidth * pressure),
      taper: tool.taper ?? state.taper,
      wobble: tool.wobble ?? state.wobble,
      dryness: tool.dryness ?? 0,
      grain: tool.grain ?? 0,
    };
  }

  function lineSetConfig(id) {
    const out = { id, ...(state.lineSets?.[id] || {}) };
    const enabledKey = `${id}Enabled`;
    const toolKey = `${id}Tool`;
    if (enabledKey in state) out.enabled = !!state[enabledKey];
    if (state[toolKey]) out.tool = state[toolKey];
    return out;
  }

  function lineSetForContourKind(kind, visible = true) {
    if (!visible) return lineSetConfig('hiddenLine');
    if (kind === 'crease') return lineSetConfig('creaseAccent');
    if (kind === 'suggestive') return lineSetConfig('suggestiveContour');
    return lineSetConfig('mainContour');
  }

  function shadowLineSet() {
    return lineSetConfig('shadowHatch');
  }

  function addMark(out, f, frame, c, tone, seed) {
    const lineSet = shadowLineSet();
    if (lineSet.enabled === false) return;
    const toolId = lineSet.tool || 'shadowPencil';
    const style=resolveStrokeStyle({toolId, lineSetId: lineSet.id, tone, seed});
    const method=state.method;
    if (method === 'stipple') { addDot(out,c,tone,seed,style,false,{toolId,lineSetId:lineSet.id,sourceType:'shadow'}); return; }
    if (method === 'halftone') { addDot(out,c,tone,seed,style,true,{toolId,lineSetId:lineSet.id,sourceType:'shadow'}); return; }
    if (method === 'scribble') { addScribble(out,f,frame,c,tone,seed,style); return; }
    if (method === 'scumble') { addScumble(out,f,frame,c,tone,seed,style); return; }
    if (method === 'graphite') {
      const n=Math.max(1, Math.round(state.layers + state.overdraw*2));
      for (let k=0;k<n;k++) addStroke(out,f,frame,c,rot2(f.flow,noise(seed,k)*.70),tone,seed+k*17,{...style, alpha:style.alpha*.55, width:style.width*.72},{lenMul:.62, curveMul:1.25});
      return;
    }
    if (method === 'drybrush') { addStroke(out,f,frame,c,f.flow,tone,seed,{...style,width:style.width*1.25},{lenMul:1.35,curveMul:1.2,dry:true}); return; }
    if (method === 'feather') { const l=norm2({x:frame.L.x,y:-frame.L.y}); addStroke(out,f,frame,c,mix2(f.flow,{x:-l.x,y:-l.y},.45),tone,seed,style,{lenMul:.78,taperMul:1.45,oneSided:true}); return; }
    if (method === 'comic') {
      addStroke(out,f,frame,c,f.flow,tone,seed,{...style,alpha:Math.min(1,style.alpha*1.25),width:style.width*(1+tone*.65)},{lenMul:1.05,taperMul:1.25});
      if (tone > .66 && hash01(seed+12) < tone) addStroke(out,f,frame,c,rot2(f.flow,deg(state.crossAngle)),tone,seed+80,{...style,width:style.width*.84},{lenMul:.75});
      return;
    }
    if (method === 'hybrid') {
      if (hash01(seed+2) < .36) addDot(out,c,tone,seed,style,false);
      addStroke(out,f,frame,c,f.flow,tone,seed,style,{lenMul:.86,curveMul:1.1});
      return;
    }
    const layerCount = method === 'crosshatch' ? Math.max(2, Math.round(state.layers)) : Math.round(state.layers);
    for (let k=0;k<layerCount;k++) {
      let dir=f.flow;
      if (method === 'crosshatch' && k>0) dir=rot2(f.flow, deg(state.crossAngle) * (k%2 ? 1 : -1) + k*.17);
      if (method === 'contourHatch') dir=mix2(f.flow, norm2({x:-(f.cy-canvas.height/2), y:f.cx-canvas.width/2}), .45);
      addStroke(out,f,frame,c,dir,tone,seed+k*97,style,{lenMul:k? .80:1, curveMul:method==='contourHatch'?1.65:1});
    }
  }

  function addDot(out,c,tone,seed,style,halftone,meta={}) {
    let r = state.dotSize * (halftone ? lerp(.55,2.0,tone) : lerp(.55,1.22,tone));
    r *= 1 + noise(seed,2) * (halftone ? .10 : .45) * state.jitter;
    out.push({kind:'dot',x:c.x,y:c.y,z:c.z,r:Math.max(.25,r),color:style.color,alpha:style.alpha,toolId:meta.toolId||style.toolId,lineSetId:meta.lineSetId||style.lineSetId,sourceType:meta.sourceType||'mark',tone,seed});
  }

  function addStroke(out,f,frame,c,dir,tone,seed,style,opt={}) {
    const lenBase = state.strokeLen * (opt.lenMul || 1) * lerp(.52,1.25,tone);
    const len = Math.max(2, lenBase * (1 + noise(seed,1) * state.lengthVar * .65));
    const curve = state.curvature * (opt.curveMul || 1);
    const steps = clamp(Math.round(len / 9) + 3, 4, 18);
    const perp={x:-dir.y,y:dir.x};
    const start=opt.oneSided ? 0 : -.5;
    const pts=[];
    const crooked = state.strokeCrookedness || 0;
    const hasKink = crooked > 0 && hash01(seed + 31.4) < state.strokeKinkChance;
    const kinkT = lerp(.22,.78,hash01(seed + 32.5));
    const kinkAmp = noise(seed, 33) * len * .18 * crooked;
    const lean = noise(seed, 34) * len * .08 * crooked;
    for (let i=0;i<steps;i++) {
      const t=i/(steps-1);
      const q=lerp(start,.5,t);
      const kink = hasKink ? (1 - clamp(Math.abs(t-kinkT)/.22,0,1)) * kinkAmp : 0;
      const crookedLean = (t-.5) * lean;
      const wob=Math.sin(t*Math.PI)*curve*state.wobble*len*.16 + noise(seed,i)*state.wobble*len*.040 + crookedLean + kink;
      const x=c.x + dir.x*len*q + perp.x*wob;
      const y=c.y + dir.y*len*q + perp.y*wob;
      let z=c.z;
      const bc=bary2({x,y},f.p[0],f.p[1],f.p[2]);
      if (state.clipToFaces && !baryInside(bc,.02)) { pts.push(null); continue; }
      if (bc) z=bc.u*f.p[0].z+bc.v*f.p[1].z+bc.w*f.p[2].z;
      if (!isVisiblePoint(frame.db,x,y,z)) { pts.push(null); continue; }
      pts.push({x,y,z,t});
    }
    for (const seg of splitSegments(pts, seed, opt.dry)) {
      if (seg.length < 2) continue;
      const width = Math.max(.15, style.width * (1 + noise(seed,7) * state.widthVar));
      const inkRamp = state.strokeToneRamp * lerp(.45,1.25,hash01(seed + 44.8)) * lerp(.45,1,tone);
      const rampDir = hash01(seed + 45.9) < .72 ? 1 : -1;
      const lengthPx = polylineLength(seg);
      if (lengthPx < (state.cleanupMinLineLengthPx || 0)) continue;
      out.push({kind:'line',pts:seg,color:style.color,alpha:style.alpha,width,taper:clamp01((style.taper ?? state.taper)*(opt.taperMul||1)),dry:!!opt.dry,seed,inkRamp,rampDir,toolId:style.toolId,lineSetId:style.lineSetId,sourceType:opt.sourceType||'shadow',tone});
    }
  }

  function polylineLength(pts) {
    let total = 0;
    for (let i = 1; i < pts.length; i++) total += Math.hypot(pts[i].x - pts[i-1].x, pts[i].y - pts[i-1].y);
    return total;
  }

  function splitSegments(pts,seed,dry) {
    const out=[]; let cur=[];
    for (let i=0;i<pts.length;i++) {
      const p=pts[i];
      const chance=state.breakup * (dry ? 1.75 : .70) * (p ? .35 + Math.sin(p.t*Math.PI) : 1);
      if (!p || (chance>0 && hash01(seed+i*41.21)<chance)) {
        if (cur.length>1) out.push(cur);
        cur=[];
      } else cur.push(p);
    }
    if (cur.length>1) out.push(cur);
    return out;
  }

  function addScribble(out,f,frame,c,tone,seed,style) {
    let dir=f.flow, x=c.x, y=c.y;
    const pts=[];
    const n=5+Math.floor(8*tone+state.overdraw*4);
    for (let i=0;i<n;i++) {
      dir=rot2(dir, noise(seed,i)*1.05);
      x += dir.x*state.strokeLen*.11 + noise(seed+3,i)*state.jitter*8;
      y += dir.y*state.strokeLen*.11 + noise(seed+5,i)*state.jitter*8;
      const bc=bary2({x,y},f.p[0],f.p[1],f.p[2]);
      if (state.clipToFaces && !baryInside(bc,.025)) { pts.push(null); continue; }
      const z=bc ? bc.u*f.p[0].z+bc.v*f.p[1].z+bc.w*f.p[2].z : c.z;
      if (!isVisiblePoint(frame.db,x,y,z)) { pts.push(null); continue; }
      pts.push({x,y,z,t:i/(n-1)});
    }
    for (const seg of splitSegments(pts,seed,false)) if (seg.length>1) out.push({kind:'line',pts:seg,color:style.color,alpha:style.alpha*.75,width:style.width*.75,taper:.12,dry:false,seed});
  }

  function addScumble(out,f,frame,c,tone,seed,style) {
    const pts=[];
    const radius=state.strokeLen*lerp(.08,.20,tone);
    const n=9+Math.floor(10*tone);
    const loops=1.0+tone*1.5;
    for (let i=0;i<n;i++) {
      const t=i/(n-1), a=t*TAU*loops + noise(seed,i)*.65;
      const x=c.x+Math.cos(a)*radius*(1+noise(seed+1,i)*.25)+noise(seed+2,i)*state.jitter*5;
      const y=c.y+Math.sin(a)*radius*.65*(1+noise(seed+3,i)*.25)+noise(seed+4,i)*state.jitter*5;
      const bc=bary2({x,y},f.p[0],f.p[1],f.p[2]);
      if (state.clipToFaces && !baryInside(bc,.025)) { pts.push(null); continue; }
      const z=bc ? bc.u*f.p[0].z+bc.v*f.p[1].z+bc.w*f.p[2].z : c.z;
      if (!isVisiblePoint(frame.db,x,y,z)) { pts.push(null); continue; }
      pts.push({x,y,z,t});
    }
    for (const seg of splitSegments(pts,seed,false)) if (seg.length>1) out.push({kind:'line',pts:seg,color:style.color,alpha:style.alpha*.58,width:style.width*.55,taper:.05,dry:false,seed});
  }

  function computeFrame() {
    const pipelineKey = buildPipelineKey(state, canvas);
    if (renderCache.frame && renderCache.pipelineKey === pipelineKey) {
      markCacheHit(perfStats);
      return renderCache.frame;
    }
    markCacheMiss(perfStats);
    const runtime = prepareMeshRuntime(state.mesh);
    const projectionContext = createFrameProjectionContext(runtime);
    const renderSelection = buildFrameRenderSelection(runtime, projectionContext);
    const frame=timeSection(perfStats, 'projection', () => transformFrame(renderSelection, projectionContext));
    frame.db=timeSection(perfStats, 'depth', () => buildDepthBuffer(frame));
    timeSection(perfStats, 'visibility', () => computeVisibilityAndFlow(frame));
    frame.contours=state.contours ? timeSection(perfStats, 'contours', () => computeContours(frame)) : [];
    frame.marks=state.shadowsEnabled ? timeSection(perfStats, 'marks', () => generateMarks(frame)) : [];
    frame.features = extractFrameFeatures(frame);
    frame.pipelineKey = pipelineKey;
    renderCache.pipelineKey = pipelineKey;
    renderCache.frame = frame;
    return frame;
  }

  async function computeFrameForRender() {
    const pipelineKey = buildPipelineKey(state, canvas);
    if (renderCache.frame && renderCache.pipelineKey === pipelineKey) {
      markCacheHit(perfStats);
      return renderCache.frame;
    }

    const mesh = state.mesh;
    const runtime = prepareMeshRuntime(mesh);
    if (!shouldUseFrameWorker(mesh, runtime)) return computeFrame();
    if (frameWorkerPending) {
      frameWorkerDropped++;
      setPerfCounter(perfStats, 'workerDropped', frameWorkerDropped);
      return null;
    }

    markCacheMiss(perfStats);
    try {
      const result = await computeFrameInWorker(mesh, runtime, pipelineKey);
      const frame = frameFromWorkerResult(result, pipelineKey);
      frame.marks = state.shadowsEnabled ? timeSection(perfStats, 'marks', () => generateMarks(frame)) : [];
      frame.features = extractFrameFeatures(frame);
      renderCache.pipelineKey = pipelineKey;
      renderCache.frame = frame;
      return frame;
    } catch (err) {
      if (err?.message === 'frame worker job superseded') return null;
      console.warn('frame worker fallback', err);
      setPerfCounter(perfStats, 'workerFallback', 1);
      return computeFrame();
    }
  }

  function ensureFrameWorker(mesh, runtime) {
    if (!frameWorker) {
      frameWorker = new Worker(new URL('./render/frameWorker.js', import.meta.url), { type: 'module' });
    }
    const nextKey = meshFrameWorkerKey(mesh);
    if (nextKey !== frameWorkerMeshKey) {
      frameWorkerMeshKey = nextKey;
      frameWorker.postMessage({
        type: 'mesh',
        meshKey: frameWorkerMeshKey,
        mesh: buildFrameWorkerMeshPayload(mesh, runtime),
      });
    }
    return frameWorker;
  }

  function computeFrameInWorker(mesh, runtime, pipelineKey) {
    const worker = ensureFrameWorker(mesh, runtime);
    const jobId = ++frameWorkerJobSeq;
    const params = snapshotFrameWorkerParams(state, canvas, {
      randomSeed: randomnessFrameSeed(),
      cameraDollyScale: cameraDollyScale(),
      cullMargin: cullMargin(),
    });

    return new Promise((resolve, reject) => {
      const pending = { jobId, pipelineKey, resolve, reject };
      frameWorkerPending = pending;
      worker.onmessage = event => {
        const msg = event.data || {};
        if (msg.type === 'mesh-ready') return;
        if (!frameWorkerPending || msg.jobId !== frameWorkerPending.jobId) return;
        frameWorkerPending = null;
        if (msg.type === 'error') reject(new Error(msg.message || 'frame worker failed'));
        else if (msg.type === 'frame') resolve(msg.result);
      };
      worker.onerror = error => {
        if (frameWorkerPending?.jobId === jobId) frameWorkerPending = null;
        reject(error);
      };
      worker.postMessage({ type: 'frame', jobId, meshKey: frameWorkerMeshKey, params });
    });
  }

  function frameFromWorkerResult(result, pipelineKey) {
    const lists = clearFrameLists(renderCache);
    const verts = lists.verts;
    const vertCount = result.verts.x.length;
    for (let i = 0; i < vertCount; i++) {
      const v = verts[i] || {};
      v.x = result.verts.x[i];
      v.y = result.verts.y[i];
      v.z = result.verts.z[i];
      v.sx = result.verts.sx[i];
      v.sy = result.verts.sy[i];
      v.inFront = Boolean(result.verts.inFront[i]);
      v.globalId = result.verts.localToGlobal ? result.verts.localToGlobal[i] : i;
      verts[i] = v;
    }
    verts.length = vertCount;

    const screenFaces = lists.screenFaces;
    const visibleFaces = lists.visibleFaces;
    const faces = lists.faces;
    faces.length = result.counters.facesTotal;
    const screen = result.screen;
    for (let i = 0; i < screen.ids.length; i++) {
      const id = screen.ids[i];
      const face = faces[id] || { p: [null, null, null], n: { x: 0, y: 0, z: 0 }, flow: { x: 1, y: 0 } };
      face.id = id;
      face.p[0] = verts[screen.a[i]];
      face.p[1] = verts[screen.b[i]];
      face.p[2] = verts[screen.c[i]];
      face.n.x = screen.nx[i];
      face.n.y = screen.ny[i];
      face.n.z = screen.nz[i];
      face.flow.x = screen.flowX[i] || 1;
      face.flow.y = screen.flowY[i] || 0;
      face.area = screen.area[i];
      face.cx = screen.cx[i];
      face.cy = screen.cy[i];
      face.depth = screen.depth[i];
      face.front = Boolean(screen.front[i]);
      face.inFront = Boolean(screen.inFront[i]);
      face.tone = screen.tone[i];
      face.ndotl = screen.ndotl[i];
      face.visible = Boolean(screen.visible[i]);
      face.visibility = screen.visibility[i];
      face.detailTier = screen.detailTier ? screen.detailTier[i] : 0;
      face.unitId = screen.unitId ? screen.unitId[i] : -1;
      face.minX = screen.minX[i];
      face.minY = screen.minY[i];
      face.maxX = screen.maxX[i];
      face.maxY = screen.maxY[i];
      face.offscreen = false;
      faces[id] = face;
      screenFaces.push(face);
      if (face.visible) visibleFaces.push(face);
    }

    const contours = contoursFromWorkerResult(result.contours, lists.contours);
    const frame = {
      verts,
      faces,
      screenFaces,
      visibleFaces,
      sortedFaces: lists.sortedFaces,
      L: result.L,
      db: result.db,
      contours,
      marks: [],
      depthMode: result.depthMode,
      cullMargin: result.viewport.margin,
      viewport: result.viewport,
      renderSelection: null,
      workerSelection: {
        selectedFaces: result.counters?.selectedFaces || result.counters?.facesSelected || 0,
        selectedEdges: result.counters?.selectedEdges || result.counters?.edgesSelected || 0,
        selectedVertices: result.counters?.selectedVertices || result.counters?.vertsSelected || 0,
      },
      pipelineKey,
    };
    frame.features = extractFrameFeatures(frame);

    for (const [name, value] of Object.entries(result.timings || {})) {
      if (name in perfStats.last) perfStats.last[name] = value;
    }
    for (const [name, value] of Object.entries(result.counters || {})) setPerfCounter(perfStats, name, value);
    setPerfCounter(perfStats, 'workerMs', result.timings?.workerTotal || 0);
    setPerfCounter(perfStats, 'workerDropped', frameWorkerDropped);
    return frame;
  }

  function contoursFromWorkerResult(contours, out = []) {
    const names = ['', 'contour', 'crease', 'suggestive'];
    out.length = contours.kind.length;
    for (let i = 0; i < contours.kind.length; i++) {
      const item = out[i] || {};
      item.x1 = contours.x1[i];
      item.y1 = contours.y1[i];
      item.z1 = contours.z1[i];
      item.x2 = contours.x2[i];
      item.y2 = contours.y2[i];
      item.z2 = contours.z2[i];
      item.kind = names[contours.kind[i]] || 'contour';
      item.visible = Boolean(contours.visible[i]);
      item.detailTier = contours.detailTier ? contours.detailTier[i] : 0;
      item.id = i;
      out[i] = item;
    }
    return out;
  }

  function extractFrameFeatures(frame) {
    const features = {
      silhouetteEdges: 0,
      boundaryEdges: 0,
      creaseEdges: 0,
      suggestiveContours: 0,
      hiddenLines: 0,
      toneBands: new Array(Math.max(1, Math.round(state.shadowBandCount || 3))).fill(0),
      shadowRegions: 0,
      highlightRegions: 0,
      highDensityAreas: 0,
      rejectedArtifacts: 0,
    };
    for (const s of frame.contours || []) {
      if (!s.visible) {
        features.hiddenLines++;
      } else if (s.kind === 'crease') {
        features.creaseEdges++;
      } else if (s.kind === 'suggestive') {
        features.suggestiveContours++;
      } else {
        features.silhouetteEdges++;
        features.boundaryEdges++;
      }
    }
    const minArea = Math.max(0, state.cleanupMinFaceAreaPx || 0);
    const bands = features.toneBands.length;
    for (const f of frame.visibleFaces || []) {
      const band = Math.min(bands - 1, Math.max(0, Math.floor(clamp01(f.tone || 0) * bands)));
      features.toneBands[band]++;
      if ((f.area || 0) < minArea * 3) features.highDensityAreas++;
    }
    for (const f of frame.screenFaces || []) {
      if (!f.visible || (f.area || 0) < minArea) features.rejectedArtifacts++;
    }
    for (const region of frame.paintRegions || []) {
      if (region.kind === 'shadow') features.shadowRegions++;
      if (region.kind === 'highlight') features.highlightRegions++;
    }
    return features;
  }

  function renderFrame(frame) {
    ctx.save();
    ctx.setTransform(1,0,0,1,0,0);
    drawCachedBackground();
    if (!state.skipSimulation) {
      if (shouldDrawPaintLayer()) drawCachedPaintLayer(frame);
      if (state.depthDebug) drawDepthDebug(frame.db);
      if (state.shadowsEnabled) drawMarks(frame.marks);
      if (state.contours) drawContours(frame.contours);
      if (state.flow) drawFlow(frame);
      if (state.seedDebug) drawSeeds(frame.marks);
      if (state.densityDebug) drawDensityDebug(frame);
      if (state.cleanupDebug) drawCleanupDebug(frame);
      if (state.regionDebug) drawRegionDebug(frame);
      if (state.visibilityDebug) drawVisibilityDebug(frame);
      if (state.detailDebug) drawDetailDebug(frame);
      if (state.budgetDebug) drawBudgetDebug(frame);
    } else {
      ctx.strokeStyle = '#2d5f62';
      ctx.lineWidth = 0.5;
      ctx.beginPath();
      for (let i = 0; i < frame.screenFaces.length; i++) {
        const f = frame.screenFaces[i];
        if (frame.depthMode === 'min' && !f.inFront) continue;
        ctx.moveTo(f.p[0].sx, f.p[0].sy);
        ctx.lineTo(f.p[1].sx, f.p[1].sy);
        ctx.lineTo(f.p[2].sx, f.p[2].sy);
        ctx.lineTo(f.p[0].sx, f.p[0].sy);
      }
      ctx.stroke();
    }
    ctx.restore();
  }

  function drawDetailDebug(frame) {
    const colors = ['rgba(0,128,255,.14)', 'rgba(0,200,100,.14)', 'rgba(255,180,0,.14)', 'rgba(255,60,0,.14)', 'rgba(0,0,0,.10)'];
    ctx.save();
    for (const f of frame.screenFaces) {
      const tier = Math.max(0, Math.min(4, f.detailTier ?? 0));
      ctx.fillStyle = colors[tier];
      facePath(f);
      ctx.fill();
    }
    ctx.restore();
  }

  function drawVisibilityDebug(frame) {
    const units = frame.renderSelection?.units || [];
    if (!units.length) return;
    ctx.save();
    ctx.lineWidth = 1;
    ctx.font = '10px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace';
    for (let i = 0; i < units.length; i++) {
      const item = units[i];
      const b = item.bounds2d;
      if (!b) continue;
      const tier = Math.max(0, Math.min(4, item.detailTier ?? 0));
      const hue = [205, 145, 42, 12, 0][tier];
      ctx.strokeStyle = `hsla(${hue}, 80%, 42%, .42)`;
      ctx.fillStyle = `hsla(${hue}, 80%, 42%, .08)`;
      const w = Math.max(0, b.maxX - b.minX);
      const h = Math.max(0, b.maxY - b.minY);
      ctx.fillRect(b.minX, b.minY, w, h);
      ctx.strokeRect(b.minX, b.minY, w, h);
      if (i < 160) {
        ctx.fillStyle = `hsla(${hue}, 80%, 22%, .85)`;
        ctx.fillText(`D${tier}`, b.minX + 3, b.minY + 3);
      }
    }
    ctx.restore();
  }

  function drawBudgetDebug(frame) {
    const c = perfStats.counters || {};
    ctx.save();
    ctx.font = '12px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace';
    ctx.fillStyle = 'rgba(255,252,245,.82)';
    ctx.fillRect(14, canvas.height - 92, 390, 72);
    ctx.fillStyle = '#17110b';
    ctx.fillText(`units ${c.sceneUnitsVisible || 0}/${c.sceneUnitsTotal || 0}`, 24, canvas.height - 68);
    ctx.fillText(`selected f/e/v ${c.facesSelected || c.selectedFaces || 0}/${c.edgesSelected || c.selectedEdges || 0}/${c.vertsSelected || c.selectedVertices || 0}`, 24, canvas.height - 48);
    ctx.fillText(`contours ${c.contoursDrawn || 0}/${c.contoursBudget || 0} marks ${c.marksGenerated || 0}/${c.marksBudget || 0}`, 24, canvas.height - 28);
    void frame;
    ctx.restore();
  }

  function shouldDrawPaintLayer() {
    return Boolean(state.paintEnabled || state.faceWash || state.tone);
  }

  function drawCachedBackground() {
    const key = buildBackgroundLayerKey(state, canvas);
    const layer = ensureLayerCanvas(renderCache.background, canvas.width, canvas.height);
    if (layer.key !== key) {
      const lctx = layer.ctx;
      lctx.setTransform(1,0,0,1,0,0);
      lctx.clearRect(0,0,layer.w,layer.h);
      lctx.fillStyle = paperColor();
      lctx.fillRect(0,0,layer.w,layer.h);
      drawPaperNoise(lctx, layer.w, layer.h);
      layer.key = key;
    }
    ctx.drawImage(layer.canvas, 0, 0);
  }

  function drawCachedPaintLayer(frame) {
    const key = buildPaintLayerKey(state, canvas, frame);
    const layer = ensureLayerCanvas(renderCache.paint, canvas.width, canvas.height);
    if (layer.key !== key) {
      timeSection(perfStats, 'paint', () => {
        const lctx = layer.ctx;
        lctx.setTransform(1,0,0,1,0,0);
        lctx.clearRect(0,0,layer.w,layer.h);
        if (state.paintEnabled) drawPaintPasses(frame, lctx);
        if (state.faceWash) drawFaceWash(frame, lctx);
        if (state.tone) drawTone(frame, lctx);
        layer.key = key;
      });
    }
    ctx.drawImage(layer.canvas, 0, 0);
  }

  function drawPaperNoise(targetCtx = ctx, width = canvas.width, height = canvas.height) {
    targetCtx.save(); targetCtx.globalAlpha=.06; targetCtx.fillStyle='#120c05';
    for (let i=0;i<110;i++) targetCtx.fillRect(hash01(i*7.1)*width, hash01(i*12.7)*height, 1, 1);
    targetCtx.restore();
  }

  function facePath(f, targetCtx = ctx) {
    targetCtx.beginPath();
    targetCtx.moveTo(f.p[0].sx,f.p[0].sy);
    targetCtx.lineTo(f.p[1].sx,f.p[1].sy);
    targetCtx.lineTo(f.p[2].sx,f.p[2].sy);
    targetCtx.closePath();
  }

  function paintFaces(frame) {
    const key = `${state.sortFaces ? 1 : 0}:${state.backface ? 1 : 0}`;
    if (frame.paintFacesKey === key && frame.paintFaces) return frame.paintFaces;
    const faces = frame.sortedFaces;
    faces.length = 0;
    for (const f of frame.visibleFaces) if (!state.backface || f.front) faces.push(f);
    if (state.sortFaces) faces.sort((a,b)=>b.depth-a.depth); // PAINT FAR TO NEAR
    frame.paintFaces = faces;
    frame.paintFacesKey = key;
    setPerfCounter(perfStats, 'paintFaces', frame.paintFaces.length);
    return frame.paintFaces;
  }

  function useBinnedPaint(faces) {
    return faces.length > 850 && typeof Path2D !== 'undefined';
  }

  function addFaceToPath(path, f) {
    path.moveTo(f.p[0].sx, f.p[0].sy);
    path.lineTo(f.p[1].sx, f.p[1].sy);
    path.lineTo(f.p[2].sx, f.p[2].sy);
    path.closePath();
  }

  function drawBinnedFacePass(faces, targetCtx, options) {
    const toneBins = options.toneBins || 18;
    const visibilityBins = options.visibilityBins || 4;
    const bucketStride = visibilityBins + 1;
    const buckets = new Array((toneBins + 1) * bucketStride);
    for (const f of faces) {
      const tone = clamp01(options.tone ? options.tone(f) : f.tone);
      if (options.skip && options.skip(f, tone)) continue;
      const toneBin = clamp(Math.round(tone * toneBins), 0, toneBins);
      const visibilityBin = clamp(Math.round(clamp01(f.visibility) * visibilityBins), 0, visibilityBins);
      const key = toneBin * bucketStride + visibilityBin;
      let bucket = buckets[key];
      if (!bucket) {
        bucket = { path: new Path2D(), count: 0, toneSum: 0, visibilitySum: 0 };
        buckets[key] = bucket;
      }
      addFaceToPath(bucket.path, f);
      bucket.count++;
      bucket.toneSum += tone;
      bucket.visibilitySum += clamp01(f.visibility);
    }
    for (const bucket of buckets) {
      if (!bucket) continue;
      const tone = bucket.toneSum / Math.max(1, bucket.count);
      const visibility = bucket.visibilitySum / Math.max(1, bucket.count);
      targetCtx.fillStyle = options.fill(tone, visibility);
      targetCtx.fill(bucket.path);
    }
  }

  function regionPath(targetCtx, region) {
    const pts = region.points || [];
    targetCtx.beginPath();
    if (!pts.length) return;
    targetCtx.moveTo(pts[0].x, pts[0].y);
    for (let i = 1; i <= pts.length; i++) {
      const p = pts[i % pts.length];
      const next = pts[(i + 1) % pts.length];
      targetCtx.quadraticCurveTo(p.x, p.y, (p.x + next.x) * .5, (p.y + next.y) * .5);
    }
    targetCtx.closePath();
  }

  function fillPaintRegion(targetCtx, region) {
    if (typeof Path2D !== 'undefined' && region.d) {
      region.path2d ||= new Path2D(region.d);
      targetCtx.fill(region.path2d);
      return;
    }
    regionPath(targetCtx, region);
    targetCtx.fill();
  }

  function clipPaintRegion(targetCtx, region) {
    if (typeof Path2D !== 'undefined' && region.d) {
      region.path2d ||= new Path2D(region.d);
      targetCtx.clip(region.path2d);
      return;
    }
    regionPath(targetCtx, region);
    targetCtx.clip();
  }

  function regionOffscreen(region, margin = 0) {
    const b = region.bounds;
    return !b || bboxOffscreen(b.minX, b.minY, b.maxX, b.maxY, margin);
  }

  function drawPaintRegions(regions, targetCtx) {
    for (const region of regions) {
      if (regionOffscreen(region, 0)) continue;
      targetCtx.save();
      targetCtx.globalCompositeOperation = region.composite || 'source-over';
      targetCtx.globalAlpha = clamp01(region.opacity);
      if (region.blur > .05) targetCtx.filter = `blur(${fmt(region.blur,2)}px)`;
      targetCtx.fillStyle = region.color;
      fillPaintRegion(targetCtx, region);
      targetCtx.restore();
    }
  }

  function clipProjectedPaintMask(targetCtx, frame, faces) {
    if (typeof Path2D !== 'undefined') {
      const key = `${faces.length}:${state.sortFaces ? 1 : 0}:${state.backface ? 1 : 0}`;
      if (frame.paintMaskKey !== key || !frame.paintMaskPath) {
        frame.paintMaskKey = key;
        frame.paintMaskPath = new Path2D();
        for (const f of faces) addFaceToPath(frame.paintMaskPath, f);
      }
      targetCtx.clip(frame.paintMaskPath);
      return;
    }
    targetCtx.beginPath();
    for (const f of faces) addFaceToPath(targetCtx, f);
    targetCtx.clip();
  }

  function drawLayeredWatercolorStrokes(regions, targetCtx) {
    if (state.paintBrush !== 'watercolor' && state.paintBrush !== 'inkWash') return;
    const wet = clamp01(state.paintWetMix ?? 0.45);
    const jitter = clamp01(state.paintRegionJitter ?? 0.25);
    targetCtx.save();
    targetCtx.globalCompositeOperation = state.paintBrush === 'inkWash' ? 'multiply' : 'source-over';
    targetCtx.lineCap = 'round';
    targetCtx.lineJoin = 'round';
    for (const region of regions) {
      if (regionOffscreen(region, 24)) continue;
      if (!region.samples || region.kind === 'highlight') continue;
      const baseAlpha = region.kind === 'base' ? 0.08 : 0.12;
      const count = clamp(Math.round(region.samples.length * lerp(0.45, 1.25, wet)), 2, 22);
      for (let i = 0; i < count; i++) {
        const sample = region.samples[i % region.samples.length];
        const seed = region.seed * 100 + i * 13.17;
        const flow = sample.flow || { x: 1, y: 0 };
        const length = lerp(20, 74, clamp01(sample.area / 1200)) * lerp(0.65, 1.25, wet);
        const width = lerp(5, 22, clamp01(sample.tone)) * (state.paintBrush === 'inkWash' ? 0.72 : 1);
        const wobble = lerp(2, 14, jitter) * (0.4 + sample.tone);
        const x0 = sample.x + noise(seed, 1) * wobble;
        const y0 = sample.y + noise(seed, 2) * wobble;
        const px = -flow.y;
        const py = flow.x;
        const x1 = x0 - flow.x * length * .48 + px * noise(seed, 3) * wobble;
        const y1 = y0 - flow.y * length * .48 + py * noise(seed, 4) * wobble;
        const x2 = x0 + flow.x * length * .48 + px * noise(seed, 5) * wobble;
        const y2 = y0 + flow.y * length * .48 + py * noise(seed, 6) * wobble;
        const cx = x0 + px * noise(seed, 7) * wobble * 1.2;
        const cy = y0 + py * noise(seed, 8) * wobble * 1.2;
        targetCtx.globalAlpha = clamp01(baseAlpha * region.opacity * lerp(0.7, 1.35, hash01(seed + 9)));
        targetCtx.strokeStyle = region.color;
        targetCtx.lineWidth = Math.max(1, width * lerp(0.55, 1.35, hash01(seed + 10)));
        targetCtx.beginPath();
        targetCtx.moveTo(x1, y1);
        targetCtx.quadraticCurveTo(cx, cy, x2, y2);
        targetCtx.stroke();
      }
    }
    targetCtx.restore();
  }

  function drawPaintPasses(frame, targetCtx = ctx) {
    const faces = paintFaces(frame);
    if (!faces.length) return;
    const shadow = hexRgb(state.paintShadowColor, '#5d6f95');
    const paper = hexRgb(state.paintPaperColor, '#f6f2e8');
    const registration = state.paintRegistration || 0;
    const regions = buildPaintRegions(frame, state);
    frame.paintRegions = regions;
    frame.features = extractFrameFeatures(frame);
    setPerfCounter(perfStats, 'paintRegions', regions.length);
    setPerfCounter(perfStats, 'paintRegionBudget', state.regionBudgetEnabled ? state.regionMaxPaintRegions : 0);
    targetCtx.save();
    clipProjectedPaintMask(targetCtx, frame, faces);
    targetCtx.translate(noise(21.7, randomnessFrameSeed()) * registration * .32, noise(31.1, randomnessFrameSeed()) * registration * .32);
    drawPaintRegions(regions, targetCtx);
    drawLayeredWatercolorStrokes(regions, targetCtx);
    if (state.paintHalftone > .01) drawPaintHalftone(regions, shadow, registration, targetCtx);
    if (state.paintGrain > .01) drawPaintGrain(regions, paper, shadow, targetCtx);
    targetCtx.restore();
  }

  function drawPaintHalftone(regions, color, registration, targetCtx = ctx) {
    targetCtx.save();
    targetCtx.globalCompositeOperation = 'multiply';
    targetCtx.translate(registration * .55, -registration * .35);
    const scale = Math.max(5, state.paintHalftoneScale || 14);
    let count = 0;
    const maxDots = state.mesh?.sourceType === 'fbx' ? 950 : 520;
    for (const region of regions) {
      if (count >= maxDots) break;
      if (regionOffscreen(region, scale)) continue;
      if (!['wash','shadow','base'].includes(region.kind)) continue;
      const bounds = region.bounds;
      const area = Math.max(1, bounds.w * bounds.h);
      const tone = clamp01(region.tone);
      if (tone < .12 && region.kind !== 'base') continue;
      const seed = (region.seed + 1) * 613.9 + shadowRandomSeed() * .41;
      const raw = area / (scale * scale * 18) * state.paintHalftone * lerp(.35, 2.0, tone) * region.opacity;
      const n = clamp(Math.floor(raw + hash01(seed)), 0, Math.min(90, maxDots - count));
      targetCtx.save();
      clipPaintRegion(targetCtx, region);
      for (let i=0;i<n;i++) {
        const p = {
          x: lerp(bounds.minX, bounds.maxX, hash01(seed + i * 3.71)),
          y: lerp(bounds.minY, bounds.maxY, hash01(seed + i * 5.19)),
        };
        const r = lerp(1.1, scale * .26, tone) * lerp(.72,1.22,hash01(seed+i*7.7));
        targetCtx.fillStyle = rgba(color, state.paintHalftone * .22 * tone * region.opacity);
        targetCtx.beginPath();
        targetCtx.arc(p.x, p.y, r, 0, TAU);
        targetCtx.fill();
        count++;
      }
      targetCtx.restore();
    }
    targetCtx.restore();
  }

  function drawPaintGrain(regions, paper, shadow, targetCtx = ctx) {
    targetCtx.save();
    const max = state.mesh?.sourceType === 'fbx' ? 520 : 320;
    const totalArea = regions.reduce((sum, region)=>sum+region.bounds.w*region.bounds.h,0) || 1;
    let count = 0;
    for (const region of regions) {
      if (count >= max) break;
      if (regionOffscreen(region, 1)) continue;
      const area = region.bounds.w * region.bounds.h;
      if (area < 8) continue;
      const seed = (region.seed + 1) * 331.7 + randomnessFrameSeed() * 11.9;
      const n = clamp(Math.round((area / totalArea) * max * state.paintGrain * 8.5 * (region.grain || 1)), 0, 40);
      targetCtx.save();
      clipPaintRegion(targetCtx, region);
      for (let i=0;i<n;i++) {
        const p = {
          x: lerp(region.bounds.minX, region.bounds.maxX, hash01(seed + i * 4.11)),
          y: lerp(region.bounds.minY, region.bounds.maxY, hash01(seed + i * 8.73)),
        };
        const c = hash01(seed + i * 17.1) > .45 ? shadow : paper;
        targetCtx.globalAlpha = state.paintGrain * lerp(.08,.24,hash01(seed+i));
        targetCtx.fillStyle = `rgb(${c.r},${c.g},${c.b})`;
        targetCtx.fillRect(p.x, p.y, 1, 1);
        count++;
      }
      targetCtx.restore();
    }
    targetCtx.restore();
  }

  function drawFaceWash(frame, targetCtx = ctx) {
    const faces=paintFaces(frame);
    targetCtx.save();
    if (useBinnedPaint(faces)) {
      drawBinnedFacePass(faces, targetCtx, {
        fill: (tone, visibility) => `rgba(23,17,11,${0.012 + tone*.052*visibility})`,
      });
    } else {
      for (const f of faces) {
        targetCtx.beginPath(); targetCtx.moveTo(f.p[0].sx,f.p[0].sy); targetCtx.lineTo(f.p[1].sx,f.p[1].sy); targetCtx.lineTo(f.p[2].sx,f.p[2].sy); targetCtx.closePath();
        targetCtx.fillStyle=`rgba(23,17,11,${0.012 + f.tone*.052*f.visibility})`; targetCtx.fill();
      }
    }
    targetCtx.restore();
  }

  function drawTone(frame, targetCtx = ctx) {
    const faces=paintFaces(frame);
    targetCtx.save();
    if (useBinnedPaint(faces)) {
      drawBinnedFacePass(faces, targetCtx, {
        fill: tone => `rgba(120,65,10,${0.05 + tone*.20})`,
      });
    } else {
      for (const f of faces) {
        targetCtx.beginPath(); targetCtx.moveTo(f.p[0].sx,f.p[0].sy); targetCtx.lineTo(f.p[1].sx,f.p[1].sy); targetCtx.lineTo(f.p[2].sx,f.p[2].sy); targetCtx.closePath();
        targetCtx.fillStyle=`rgba(120,65,10,${0.05 + f.tone*.20})`; targetCtx.fill();
      }
    }
    targetCtx.restore();
  }

  function drawMarks(marks) {
    ctx.save(); ctx.lineCap='round'; ctx.lineJoin='round';
    const inkAlpha = clamp(state.inkDominance || 1, .35, 1.35);
    const inkWidth = lerp(.92, 1.10, clamp01((inkAlpha - .35) / 1));
    let dotBatch = null;
    const flushDots = () => {
      if (!dotBatch) return;
      ctx.globalAlpha = dotBatch.alpha;
      ctx.fillStyle = dotBatch.color;
      ctx.fill(dotBatch.path);
      dotBatch = null;
    };
    for (const m of marks) {
      if (m.kind === 'dot') {
        const alpha = clamp01(m.alpha * inkAlpha);
        if (typeof Path2D === 'undefined') {
          flushDots();
          ctx.globalAlpha=alpha; ctx.fillStyle=m.color; ctx.beginPath(); ctx.arc(m.x,m.y,m.r,0,TAU); ctx.fill();
          continue;
        }
        if (!dotBatch || dotBatch.color !== m.color || dotBatch.alpha !== alpha) {
          flushDots();
          dotBatch = {color:m.color, alpha, path:new Path2D()};
        }
        dotBatch.path.moveTo(m.x + m.r, m.y);
        dotBatch.path.arc(m.x,m.y,m.r,0,TAU);
      } else if (m.pts.length > 1) {
        flushDots();
        ctx.strokeStyle=m.color; ctx.lineWidth=m.width * inkWidth;
        ctx.setLineDash(m.dry ? [Math.max(1,m.width*1.2), Math.max(2,m.width*2.4)] : []);
        if (m.inkRamp > .01 && m.pts.length > 2) drawVariableStroke({...m, alpha:clamp01(m.alpha * inkAlpha), width:m.width * inkWidth}, m.pts);
        else { ctx.globalAlpha=clamp01(m.alpha * inkAlpha); strokeSmooth(m.pts); }
        if (m.taper > .05 && m.pts.length > 3) {
          ctx.lineWidth=Math.max(.15,m.width*(1-m.taper*.55) * inkWidth); ctx.setLineDash([]);
          if (m.inkRamp > .01) drawVariableStroke({...m, alpha:clamp01(m.alpha*.28 * inkAlpha), width:ctx.lineWidth}, m.pts.slice(1,-1));
          else { ctx.globalAlpha=clamp01(m.alpha*.28 * inkAlpha); strokeSmooth(m.pts.slice(1,-1)); }
        }
      }
    }
    flushDots();
    ctx.setLineDash([]); ctx.restore();
  }

  function drawVariableStroke(mark, pts) {
    for (let i=0;i<pts.length-1;i++) {
      const a=pts[i], b=pts[i+1];
      const ta = a.t ?? i/(pts.length-1);
      const tb = b.t ?? (i+1)/(pts.length-1);
      const t = (ta + tb) * .5;
      const rampT = mark.rampDir >= 0 ? t : 1-t;
      const ramp = lerp(1 - mark.inkRamp*.45, 1 + mark.inkRamp*.55, rampT);
      ctx.globalAlpha=clamp(mark.alpha * ramp, .015, 1);
      ctx.lineWidth=Math.max(.15, mark.width * (1 + noise(mark.seed || 0, i + 101) * state.widthVar * .20));
      ctx.beginPath(); ctx.moveTo(a.x,a.y); ctx.lineTo(b.x,b.y); ctx.stroke();
    }
  }

  function strokeSmooth(pts) {
    ctx.beginPath(); ctx.moveTo(pts[0].x, pts[0].y);
    if (pts.length === 2) ctx.lineTo(pts[1].x, pts[1].y);
    else {
      for (let i=1;i<pts.length-1;i++) ctx.quadraticCurveTo(pts[i].x,pts[i].y,(pts[i].x+pts[i+1].x)/2,(pts[i].y+pts[i+1].y)/2);
      const last=pts[pts.length-1]; ctx.lineTo(last.x,last.y);
    }
    ctx.stroke();
  }

  function contourFrameSeed() {
    if (!state.contourHumanize) return 0;
    return Math.floor(state.cameraYaw * .18 * clamp01(state.contourFrameVariance));
  }

  function contourVariantPoints(s) {
    const len = Math.hypot(s.x2-s.x1, s.y2-s.y1);
    if (!state.contourHumanize || s.kind !== 'contour' || len < 4) return [{x:s.x1,y:s.y1},{x:s.x2,y:s.y2}];
    const seed = (s.id + 1) * 817.37 + contourFrameSeed() * 53.19;
    if (state.contourGaps > 0 && hash01(seed + 3.7) < state.contourGaps * .42) return null;
    const nx = -(s.y2-s.y1) / len;
    const ny = (s.x2-s.x1) / len;
    const drift = state.contourDrift * (state.modelSource === 'walking' ? 1.15 : 1);
    const wobble = state.contourWobble;
    const o0 = noise(seed, 1) * drift;
    const o1 = noise(seed, 2) * drift;
    const mid = noise(seed, 3) * drift * (1 + wobble * 1.8);
    const insetA = state.contourGaps > 0 && hash01(seed + 11.1) < state.contourGaps ? lerp(.04,.18,hash01(seed+12.4)) : 0;
    const insetB = state.contourGaps > 0 && hash01(seed + 15.2) < state.contourGaps ? lerp(.04,.18,hash01(seed+16.5)) : 0;
    const x1 = lerp(s.x1, s.x2, insetA), y1 = lerp(s.y1, s.y2, insetA);
    const x2 = lerp(s.x2, s.x1, insetB), y2 = lerp(s.y2, s.y1, insetB);
    return [
      {x:x1 + nx*o0, y:y1 + ny*o0},
      {x:lerp(x1,x2,.5) + nx*mid, y:lerp(y1,y2,.5) + ny*mid},
      {x:x2 + nx*o1, y:y2 + ny*o1}
    ];
  }

  function strokeContourVariant(s) {
    const len = Math.hypot(s.x2 - s.x1, s.y2 - s.y1);
    if (!state.contourHumanize || s.kind !== 'contour' || len < 4) {
      ctx.beginPath();
      ctx.moveTo(s.x1, s.y1);
      ctx.lineTo(s.x2, s.y2);
      ctx.stroke();
      return true;
    }
    const seed = (s.id + 1) * 817.37 + contourFrameSeed() * 53.19;
    if (state.contourGaps > 0 && hash01(seed + 3.7) < state.contourGaps * .42) return false;
    const nx = -(s.y2 - s.y1) / len;
    const ny = (s.x2 - s.x1) / len;
    const drift = state.contourDrift * (state.modelSource === 'walking' ? 1.15 : 1);
    const wobble = state.contourWobble;
    const o0 = noise(seed, 1) * drift;
    const o1 = noise(seed, 2) * drift;
    const mid = noise(seed, 3) * drift * (1 + wobble * 1.8);
    const insetA = state.contourGaps > 0 && hash01(seed + 11.1) < state.contourGaps ? lerp(.04, .18, hash01(seed + 12.4)) : 0;
    const insetB = state.contourGaps > 0 && hash01(seed + 15.2) < state.contourGaps ? lerp(.04, .18, hash01(seed + 16.5)) : 0;
    const x1 = lerp(s.x1, s.x2, insetA);
    const y1 = lerp(s.y1, s.y2, insetA);
    const x2 = lerp(s.x2, s.x1, insetB);
    const y2 = lerp(s.y2, s.y1, insetB);
    const ax = x1 + nx * o0;
    const ay = y1 + ny * o0;
    const mx = lerp(x1, x2, .5) + nx * mid;
    const my = lerp(y1, y2, .5) + ny * mid;
    const bx = x2 + nx * o1;
    const by = y2 + ny * o1;
    ctx.beginPath();
    ctx.moveTo(ax, ay);
    ctx.quadraticCurveTo(mx, my, (mx + bx) / 2, (my + by) / 2);
    ctx.lineTo(bx, by);
    ctx.stroke();
    return true;
  }

  function drawContours(segs) {
    ctx.save(); ctx.lineCap='round'; ctx.lineJoin='round';
    const inkAlpha = clamp(state.inkDominance || 1, .35, 1.35);
    const inkWidth = lerp(.92, 1.12, clamp01((inkAlpha - .35) / 1));
    for (const s of segs) {
      const lineSet = lineSetForContourKind(s.kind, s.visible);
      if (lineSet.enabled === false) continue;
      const len = Math.hypot(s.x2 - s.x1, s.y2 - s.y1);
      if (len < (lineSet.minLengthPx || state.cleanupMinLineLengthPx || 0)) continue;
      const style = resolveStrokeStyle({
        toolId: lineSet.tool || 'mainInk',
        lineSetId: lineSet.id,
        tone: s.kind === 'contour' ? 1 : (lineSet.strength || .55),
        seed: (s.id + 1) * 19.23 + contourFrameSeed()
      });
      ctx.globalAlpha=clamp01(style.alpha * (s.visible ? 1 : .55) * inkAlpha);
      ctx.strokeStyle=style.color;
      const widthNoise = state.contourHumanize && s.kind==='contour' ? lerp(.88,1.18,hash01((s.id+1)*19.23 + contourFrameSeed())) : 1;
      ctx.lineWidth=style.width * widthNoise * inkWidth;
      ctx.setLineDash(s.visible ? [] : [6,5]);
      strokeContourVariant(s);
    }
    ctx.restore();
  }

  function drawFlow(frame) {
    ctx.save(); ctx.strokeStyle='rgba(20,94,103,.68)'; ctx.lineWidth=1; ctx.lineCap='round';
    let k=0;
    for (const f of frame.visibleFaces) {
      if (!f.visible || f.area < 20 || (++k % 3)) continue;
      const l=12, d=f.flow; ctx.beginPath(); ctx.moveTo(f.cx-d.x*l*.5,f.cy-d.y*l*.5); ctx.lineTo(f.cx+d.x*l*.5,f.cy+d.y*l*.5); ctx.stroke();
    }
    ctx.restore();
  }

  function drawSeeds(marks) {
    ctx.save(); ctx.globalAlpha=.55; ctx.fillStyle='#a94917';
    for (const m of marks) { const p=m.kind==='dot'?m:m.pts[0]; if (p) ctx.fillRect(p.x-1,p.y-1,2,2); }
    ctx.restore();
  }

  function drawDensityDebug(frame) {
    const faces = frame.visibleFaces || [];
    if (!faces.length) return;
    let maxArea = 0;
    for (const f of faces) maxArea = Math.max(maxArea, f.area || 0);
    const areaSpan = Math.max(1, maxArea);
    ctx.save();
    ctx.globalCompositeOperation = 'source-over';
    for (const f of faces) {
      if (!f.visible) continue;
      const density = 1 - clamp01((f.area || 0) / areaSpan);
      if (density < .12) continue;
      ctx.fillStyle = `rgba(${Math.round(255 * density)}, ${Math.round(180 * (1 - density))}, 24, ${0.12 + density * .34})`;
      ctx.beginPath();
      ctx.moveTo(f.p[0].sx, f.p[0].sy);
      ctx.lineTo(f.p[1].sx, f.p[1].sy);
      ctx.lineTo(f.p[2].sx, f.p[2].sy);
      ctx.closePath();
      ctx.fill();
    }
    ctx.restore();
  }

  function drawCleanupDebug(frame) {
    const minArea = state.cleanupMinFaceAreaPx || 0;
    const maxEdge = state.cleanupMaxEdgeLengthPx || Infinity;
    ctx.save();
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.strokeStyle = 'rgba(220, 54, 36, .72)';
    ctx.fillStyle = 'rgba(220, 54, 36, .10)';
    let drawn = 0;
    for (const f of frame.screenFaces || []) {
      if (drawn > 900) break;
      const e0 = Math.hypot(f.p[0].sx - f.p[1].sx, f.p[0].sy - f.p[1].sy);
      const e1 = Math.hypot(f.p[1].sx - f.p[2].sx, f.p[1].sy - f.p[2].sy);
      const e2 = Math.hypot(f.p[2].sx - f.p[0].sx, f.p[2].sy - f.p[0].sy);
      const rejected = (f.area || 0) < minArea || Math.max(e0, e1, e2) > maxEdge || !f.visible;
      if (!rejected) continue;
      ctx.beginPath();
      ctx.moveTo(f.p[0].sx, f.p[0].sy);
      ctx.lineTo(f.p[1].sx, f.p[1].sy);
      ctx.lineTo(f.p[2].sx, f.p[2].sy);
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
      drawn++;
    }
    ctx.restore();
  }

  function drawRegionDebug(frame) {
    const regions = frame.paintRegions || buildPaintRegions(frame, state);
    if (!regions.length) return;
    ctx.save();
    ctx.font = '11px ui-monospace, SFMono-Regular, Consolas, monospace';
    ctx.textBaseline = 'top';
    ctx.lineWidth = 1;
    for (let i = 0; i < regions.length; i++) {
      const region = regions[i];
      const b = region.bounds;
      if (!b) continue;
      ctx.strokeStyle = region.kind === 'shadow' ? 'rgba(84, 57, 184, .72)' : region.kind === 'highlight' ? 'rgba(232, 144, 36, .82)' : 'rgba(28, 111, 132, .68)';
      ctx.fillStyle = 'rgba(255, 255, 255, .78)';
      if (region.d && typeof Path2D !== 'undefined') ctx.stroke(new Path2D(region.d));
      ctx.strokeRect(b.minX, b.minY, b.w, b.h);
      const label = `${region.kind}:${i}`;
      ctx.fillRect(b.minX, b.minY, ctx.measureText(label).width + 6, 15);
      ctx.fillStyle = '#241810';
      ctx.fillText(label, b.minX + 3, b.minY + 2);
    }
    ctx.restore();
  }

  function drawDepthDebug(db) {
    const img=ctx.createImageData(db.w,db.h); let min=1e9,max=-1e9;
    const aliveDepth = db.nearIsSmaller
      ? z => z < 1e8
      : z => z > -1e8;
    for (const z of db.depth) if (aliveDepth(z)) { min=Math.min(min,z); max=Math.max(max,z); }
    const span=Math.max(.001,max-min);
    for (let i=0;i<db.depth.length;i++) {
      const z=db.depth[i], alive=aliveDepth(z), t=alive?clamp01((z-min)/span):0;
      const val = Math.floor((1-t)*255);
      img.data[i*4]=val; img.data[i*4+1]=val; img.data[i*4+2]=val; img.data[i*4+3]=alive?255:0;
    }
    const tmp=document.createElement('canvas'); tmp.width=db.w; tmp.height=db.h; tmp.getContext('2d').putImageData(img,0,0);
    ctx.drawImage(tmp,0,0,canvas.width,canvas.height);
  }

  function updateStatus(frame) {
    const visible=frame.visibleFaces.filter(f=>!state.backface || f.front).length;
    const model = state.mesh?.name || 'model';
    const tween = '';
    const topology = state.mesh?.sourceType === 'fbx' ? ` · topology: cached` : '';
    const fbxSource = escapeHtml(state.fbxSourceLabel || 'walking.fbx');
    const anim = state.modelSource === 'walking' ? ` · FBX source: <b>${state.walkingFbxReady?fbxSource:'not found'}</b>${topology}` : '';
    const features = frame.features || extractFrameFeatures(frame);
    const html = `model: <b>${escapeHtml(model)}</b>${anim}<br>` +
      `projection: <b>${state.projectionMode}</b> · focal length: <b>${Math.round(state.focalLength)}mm</b><br>` +
      `camera xyz: ${fmt(state.cameraX,2)}, ${fmt(state.cameraY,2)}, ${fmt(state.cameraZ,2)} · look: ${Math.round(state.cameraYaw)}°/${Math.round(state.cameraPitch)}°<br>` +
      `faces: ${visible}/${frame.screenFaces.length}/${frame.faces.length} visible/screen/total · strokes: ${frame.marks.length}<br>` +
      `features: contour ${features.silhouetteEdges}, crease ${features.creaseEdges}, suggestive ${features.suggestiveContours}, hidden ${features.hiddenLines}, regions ${features.shadowRegions + features.highlightRegions}<br>` +
      formatPerfStats(perfStats, fmt);
    if (statusEl) statusEl.innerHTML = html;
  }

  function scheduleRender(scope = DIRTY_FLAGS.PROJECTION) {
    markDirty(dirtyFlags, scope);
    state.needsRender = true;
    if (state.scheduled) return;
    state.scheduled = true;
    requestAnimationFrame(async () => {
      state.scheduled = false;
      if (!state.mesh || !state.needsRender) return;
      state.needsRender = false;
      try {
        const totalStart = performance.now();
        resetPerfFrame(perfStats, false);
        const frame = await computeFrameForRender();
        if (!frame) {
          state.needsRender = true;
          return;
        }
        if (frame.pipelineKey !== buildPipelineKey(state, canvas)) {
          scheduleRender(DIRTY_FLAGS.PROJECTION);
          return;
        }
        state.frame = frame;
        const drawStart = performance.now();
        if (dirtyFlags.projection || dirtyFlags.visibility || dirtyFlags.paint) renderCache.paint.key = '';
        if (dirtyFlags.mesh || dirtyFlags.projection || dirtyFlags.visibility || dirtyFlags.npr || dirtyFlags.paint || dirtyFlags.display) {
          renderCache.svg.key = '';
          renderCache.svg.text = '';
        }
        renderFrame(frame);
        timeSectionEnd(perfStats, 'draw', drawStart);
        finishPerfFrame(perfStats, totalStart);
        updateStatus(frame);
        clearDirty(dirtyFlags);
        state.lastError = '';
      } catch (err) {
        state.lastError = err && err.stack ? err.stack : String(err);
        console.error(err);
      }
    });
  }

  function updateCameraFromKeys(dt) {
    if (!updateCameraFromKeysForState(state, dt)) return false;
    syncUi();
    scheduleRender();
    return true;
  }

  function resetViewForModel(kind = 'obj') {
    state.pressedKeys.clear();
    state.auto = false;
    if (kind === 'fbx') {
      Object.assign(state, {
        controlMode: 'freelook',
        angleSnap: 0,
        yaw: -24,
        pitch: 12,
        zoom: 1,
        cameraYaw: 0,
        cameraPitch: 0,
        cameraX: 0,
        cameraY: 0.6,
        cameraZ: 6.5,
        rawYaw: -24,
        rawPitch: 12,
        rawCameraYaw: 0,
        rawCameraPitch: 0,
        focalLength: 35,
        projectionMode: 'perspective',
      });
    } else {
      Object.assign(state, {
        controlMode: 'orbit',
        angleSnap: 0,
        yaw: -24,
        pitch: 12,
        zoom: 1,
        cameraYaw: 0,
        cameraPitch: 0,
        cameraX: 0,
        cameraY: 0,
        cameraZ: 0,
        rawYaw: -24,
        rawPitch: 12,
        rawCameraYaw: 0,
        rawCameraPitch: 0,
        focalLength: 35,
        projectionMode: 'perspective',
      });
    }
  }

  function snapshotSettings() {
    const values = {};
    for (const key of persistedFields) values[key] = state[key];
    return { version: 7, savedAt: Date.now(), values };
  }

  function requestSaveSettings() {
    if (!settingsLoaded) return;
    clearTimeout(saveSettingsTimer);
    saveSettingsTimer = setTimeout(saveSettings, 240);
  }

  function saveSettings() {
    if (!settingsLoaded) return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshotSettings()));
    } catch (err) {
      console.warn('Settings save failed', err);
    }
  }

  function applyPaintPalette(id, save = true) {
    const palette = paintPalettes[id] || paintPalettes.cleanComic;
    state.paintPalette = paintPalettes[id] ? id : 'cleanComic';
    state.paintPaperColor = palette.paper;
    state.paintBaseColor = palette.base;
    state.paintShadowColor = palette.shadow;
    state.paintHighlightColor = palette.highlight;
    syncUi();
    scheduleRender(DIRTY_FLAGS.PAINT);
    if (save) requestSaveSettings();
  }

  function loadSavedSettings() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) return false;
      const parsed = JSON.parse(raw);
      const values = parsed?.values;
      if (!values || typeof values !== 'object') return false;
      for (const key of persistedFields) {
        if (!(key in values)) continue;
        const value = values[key];
        if (key in rangeControls) {
          const num = Number(value);
          if (Number.isFinite(num)) state[key] = num;
        } else if (colorControls.includes(key)) {
          state[key] = normalizeHexColor(value, state[key]);
        } else if (checkControls.includes(key)) {
          state[key] = !!value;
        } else if (typeof value === 'string') {
          state[key] = value;
        }
      }
      state.controlMode = state.controlMode === 'freelook' ? 'freelook' : 'orbit';
      state.projectionMode = state.projectionMode === 'ortho' ? 'ortho' : 'perspective';
      return true;
    } catch (err) {
      return false;
    }
  }

  function syncUi() {
    for (const [id, [key, format]] of Object.entries(rangeControls)) {
      const el=$(id), label=$(id+'V');
      if (el && String(el.value) !== String(state[key])) el.value=state[key];
      if (label) label.textContent=format(state[key]);
    }
    if ($('controlMode')) $('controlMode').value=state.controlMode;
    if ($('projectionMode')) $('projectionMode').value=state.projectionMode;
    for (const id of checkControls) { const el=$(id); if (el) el.checked=!!state[id]; }
    if ($('method')) $('method').value=state.method;
    if ($('flowMode')) $('flowMode').value=state.flowMode;
    for (const id of lineToolControls) { const el=$(id); if (el) el.value=state[id]; }
    if ($('preset')) $('preset').value=state.preset;
    if ($('paintBrush')) $('paintBrush').value=state.paintBrush;
    if ($('paintPalette')) $('paintPalette').value=state.paintPalette;
    for (const id of colorControls) { const el=$(id); if (el) el.value=normalizeHexColor(state[id], el.value || '#000000'); }
    if ($('modelSource')) $('modelSource').value=state.modelSource;
    if ($('animFps')) $('animFps').value=String(state.animFps);
    for (const b of document.querySelectorAll('.modeButtons button')) b.classList.toggle('active', b.dataset.mode === state.mode);
  }

  function bindUi() {
    for (const [id, [key, format]] of Object.entries(rangeControls)) {
      const el=$(id), label=$(id+'V');
      if (!el) continue;
      const update = () => {
        const value = Number(el.value);
        if (key === 'yaw' || key === 'pitch') setModelAngles(key === 'yaw' ? value : state.yaw, key === 'pitch' ? value : state.pitch);
        else if (key === 'cameraYaw' || key === 'cameraPitch') setCameraAngles(key === 'cameraYaw' ? value : state.cameraYaw, key === 'cameraPitch' ? value : state.cameraPitch);
        else state[key] = value;
        if (key === 'angleSnap') applyAngleSnap();
        if (label) label.textContent = format(state[key]);
        syncUi();
        scheduleRender(renderScopeForKey(key));
        requestSaveSettings();
      };
      el.addEventListener('input', update);
      el.addEventListener('change', update);
    }
    for (const id of checkControls) {
      const el=$(id); if (!el) continue;
      el.addEventListener('change', () => { state[id]=el.checked; scheduleRender(renderScopeForKey(id)); requestSaveSettings(); });
    }
    for (const id of colorControls) {
      const el=$(id); if (!el) continue;
      el.addEventListener('input', () => { state[id]=normalizeHexColor(el.value, state[id]); scheduleRender(DIRTY_FLAGS.PAINT); requestSaveSettings(); });
      el.addEventListener('change', () => { state[id]=normalizeHexColor(el.value, state[id]); syncUi(); scheduleRender(DIRTY_FLAGS.PAINT); requestSaveSettings(); });
    }
    $('method')?.addEventListener('change', e => { state.method=e.target.value; scheduleRender(DIRTY_FLAGS.NPR); requestSaveSettings(); });
    $('flowMode')?.addEventListener('change', e => { state.flowMode=e.target.value; scheduleRender(DIRTY_FLAGS.NPR); requestSaveSettings(); });
    for (const id of lineToolControls) {
      const el=$(id); if (!el) continue;
      el.addEventListener('change', () => { state[id]=el.value; scheduleRender(DIRTY_FLAGS.NPR); requestSaveSettings(); });
    }
    $('preset')?.addEventListener('change', e => applyPreset(e.target.value));
    $('paintBrush')?.addEventListener('change', e => { state.paintBrush=e.target.value; syncUi(); scheduleRender(DIRTY_FLAGS.PAINT); requestSaveSettings(); });
    $('paintPalette')?.addEventListener('change', e => applyPaintPalette(e.target.value));
    $('controlMode')?.addEventListener('change', e => { state.controlMode=e.target.value === 'freelook' ? 'freelook' : 'orbit'; syncUi(); scheduleRender(DIRTY_FLAGS.PROJECTION); requestSaveSettings(); });
    $('projectionMode')?.addEventListener('change', e => { state.projectionMode=e.target.value === 'perspective' ? 'perspective' : 'ortho'; syncUi(); scheduleRender(DIRTY_FLAGS.PROJECTION); requestSaveSettings(); });
    $('modelSource')?.addEventListener('change', e => setModelSource(e.target.value));
    $('animFps')?.addEventListener('change', e => { state.animFps=Number(e.target.value)||24; state.animAccumulator=0; requestSaveSettings(); });
    for (const b of document.querySelectorAll('.modeButtons button')) b.addEventListener('click', () => { state.mode=b.dataset.mode; syncUi(); scheduleRender(DIRTY_FLAGS.NPR); requestSaveSettings(); });
    for (const b of document.querySelectorAll('.tabBtn')) b.addEventListener('click', () => {
      document.querySelectorAll('.tabBtn').forEach(x=>x.classList.toggle('active', x===b));
      document.querySelectorAll('.tabPane').forEach(p=>p.classList.toggle('active', p.id===b.dataset.tab));
    });
    $('reset')?.addEventListener('click', () => { resetViewForModel(builtinModelMap.get(state.modelSource)?.kind || 'obj'); syncUi(); scheduleRender(); requestSaveSettings(); });
    $('exportSvg')?.addEventListener('click', exportSvg);
    $('exportPng')?.addEventListener('click', exportPng);
    $('exportAtlas')?.addEventListener('click', exportAtlas);
    $('exportFbxClip')?.addEventListener('click', exportFbxClip);
    $('file')?.addEventListener('change', e => loadFile(e.target.files && e.target.files[0]));
    $('fbxFile')?.addEventListener('change', e => loadCustomFbxFile(e.target.files && e.target.files[0]));
    canvas.addEventListener('contextmenu', e => e.preventDefault());
    canvas.addEventListener('pointerdown', e => {
      state.dragging=true;
      state.lastX=e.clientX;
      state.lastY=e.clientY;
      state.auto=false;
      state.dragMode = (e.button === 1 || e.altKey)
        ? 'pan'
        : (state.controlMode === 'freelook' ? 'camera' : (e.button === 2 || e.shiftKey ? 'camera' : 'model'));
      canvas.setPointerCapture?.(e.pointerId);
      syncUi();
    });
    canvas.addEventListener('pointermove', e => {
      if (!state.dragging) return;
      const dx=e.clientX-state.lastX, dy=e.clientY-state.lastY;
      state.lastX=e.clientX; state.lastY=e.clientY;
      if (state.dragMode === 'camera') {
        setCameraAngles(state.cameraYaw - dx*.28, state.cameraPitch - dy*.24);
      } else if (state.dragMode === 'model') {
        setModelAngles((state.rawYaw ?? state.yaw) + dx*.45, (state.rawPitch ?? state.pitch) + dy*.35);
      } else if (state.dragMode === 'pan') {
        const pan = 0.01;
        state.cameraX -= dx * pan;
        state.cameraY += dy * pan;
      }
      syncUi();
      scheduleRender();
    });
    canvas.addEventListener('pointerup', e => { state.dragging=false; requestSaveSettings(); try{canvas.releasePointerCapture?.(e.pointerId);}catch(_){} });
    canvas.addEventListener('pointercancel', e => { state.dragging=false; requestSaveSettings(); try{canvas.releasePointerCapture?.(e.pointerId);}catch(_){} });
    canvas.addEventListener('wheel', e => {
      e.preventDefault();
      if (state.controlMode === 'freelook') state.cameraZ=clamp(state.cameraZ + e.deltaY*.01, -100, 100);
      else state.zoom = clamp(state.zoom * Math.exp(-e.deltaY*.001), .55, 1.8);
      syncUi();
      scheduleRender();
      requestSaveSettings();
    }, {passive:false});
    window.addEventListener('resize', () => { resizeCanvas(); scheduleRender(); });
    window.addEventListener('keydown', e => {
      if (isTypingTarget(e.target)) return;
      if (cameraKeyCodes.has(e.code)) {
        e.preventDefault();
        state.pressedKeys.add(e.code);
        state.auto=false;
        syncUi();
        return;
      }
      if (e.code==='Space') { e.preventDefault(); state.auto=!state.auto; syncUi(); scheduleRender(); requestSaveSettings(); }
    });
    window.addEventListener('keyup', e => { state.pressedKeys.delete(e.code); if (cameraKeyCodes.has(e.code)) requestSaveSettings(); });
    window.addEventListener('blur', () => { if (state.pressedKeys.size) requestSaveSettings(); state.pressedKeys.clear(); });
    window.addEventListener('dragover', e => e.preventDefault());
    window.addEventListener('drop', e => { e.preventDefault(); loadFile(e.dataTransfer.files && e.dataTransfer.files[0]); });
  }

  function applyPreset(key) {
    const p=presets[key]; if (!p) return;
    Object.assign(state,p);
    syncLineSetControlFields();
    state.preset=key;
    syncUi();
    scheduleRender();
    requestSaveSettings();
  }

  function loadFile(file) {
    if (!file) return;
    const lowerName = String(file.name || '').toLowerCase();
    if (lowerName.endsWith('.fbx')) {
      loadCustomFbxFile(file);
      return;
    }
    loadOBJSource({ file }, file.name || 'dropped OBJ');
  }

  function syncLineSetControlFields() {
    for (const [lineSetId, config] of Object.entries(state.lineSets || {})) {
      const enabledKey = `${lineSetId}Enabled`;
      const toolKey = `${lineSetId}Tool`;
      if (enabledKey in state) state[enabledKey] = config.enabled !== false;
      if (toolKey in state && config.tool) state[toolKey] = config.tool;
    }
  }

  async function parseOBJAsync(source, name) {
    if (typeof Worker === 'undefined') {
      let text = source.text || '';
      if (!text && source.file && typeof source.file.text === 'function') text = await source.file.text();
      if (!text && source.url) {
        const response = await fetch(source.url);
        if (!response.ok) throw new Error(`${name || source.url} fetch failed: ${response.status}`);
        text = await response.text();
      }
      const mesh = parseOBJ(text);
      prepareMeshRuntime(mesh);
      return { mesh, sourceLength: text.length };
    }
    const id = ++objParseSeq;
    if (!objParseWorker) {
      objParseWorker = new Worker(new URL('./mesh/objParseWorker.js', import.meta.url), { type: 'module' });
    }

    return new Promise((resolve, reject) => {
      const worker = objParseWorker;
      const cleanup = () => {
        worker.removeEventListener('message', onMessage);
        worker.removeEventListener('error', onError);
      };
      const onMessage = event => {
        const data = event.data || {};
        if (data.id !== id) return;
        cleanup();
        if (data.ok) resolve({ mesh: data.mesh, sourceLength: data.sourceLength || 0 });
        else reject(new Error(data.error || 'OBJ parser worker failed.'));
      };
      const onError = error => {
        cleanup();
        if (objParseWorker === worker) {
          objParseWorker.terminate();
          objParseWorker = null;
        }
        reject(error instanceof Error ? error : new Error(String(error?.message || error)));
      };
      worker.addEventListener('message', onMessage);
      worker.addEventListener('error', onError);
      worker.postMessage({ id, ...source, name });
    });
  }

  function clearCustomFbxUrl() {
    if (!state.fbxCustomObjectUrl) return;
    URL.revokeObjectURL(state.fbxCustomObjectUrl);
    state.fbxCustomObjectUrl = '';
  }

  function resetFbxRuntime() {
    objLoadToken++;
    state.fbxLoadToken = (state.fbxLoadToken || 0) + 1;
    state.fbxRuntime = null;
    state.fbxLoadPromise = null;
    state.fbxError = '';
    state.fbxAdapterKind = '';
    state.frame = null;
    renderCache.frame = null;
    renderCache.pipelineKey = '';
    invalidateDerivedCaches(renderCache);
  }

  async function loadCustomFbxFile(file) {
    if (!file) return;
    clearCustomFbxUrl();
    state.fbxCustomObjectUrl = URL.createObjectURL(file);
    state.fbxSourceUrl = state.fbxCustomObjectUrl;
    state.fbxSourceLabel = file.name || 'custom.fbx';
    state.walkingFbxReady = true;
    resetFbxRuntime();
    resetViewForModel('fbx');
    state.modelSource = 'walking';
    syncUi();
    requestSaveSettings();
    await startFbxMode();
  }

  function loadOBJText(text, name) {
    return loadOBJSource({ text }, name);
  }

  async function loadOBJSource(source, name) {
    const token = ++objLoadToken;
    try {
      if (statusEl) statusEl.textContent = `loading ${name || 'OBJ'}...`;
      const parsed = await parseOBJAsync(source, name);
      if (token !== objLoadToken) return;
      const mesh = parsed.mesh;
      prepareMeshRuntime(mesh);
      mesh.name = name;
      mesh.cacheId = `obj:${++meshRevision}:${name}:${parsed.sourceLength || 0}`;
      mesh.frameVersion = 0;
      resetViewForModel('obj');
      state.mesh = mesh;
      state.frame = null;
      renderCache.frame = null;
      renderCache.pipelineKey = '';
      invalidateDerivedCaches(renderCache);
      scheduleRender(DIRTY_FLAGS.MESH);
    }
    catch(err) {
      if (token !== objLoadToken) return;
      console.error(err);
      if (statusEl) statusEl.textContent = 'OBJ error: ' + (err && err.message ? err.message : err);
    }
  }

  function populateModelSourceOptions() {
    const select = $('modelSource');
    if (!select) return;
    select.innerHTML = BUILTIN_MODELS.map(model => `<option value="${model.id}">${escapeHtml(model.label)}</option>`).join('');
    if (!builtinModelMap.has(state.modelSource)) state.modelSource = BUILTIN_MODELS[0]?.id || 'suzanne';
    select.value = state.modelSource;
  }

  async function loadBuiltInObjModel(model) {
    try {
      await loadOBJSource({ url: model.url }, model.label);
    } catch (err) {
      console.error(err);
    }
  }

  async function ensureFbxRuntime() {
    return ensureFbxRuntimeForState(state);
  }
  function setModelSource(source) {
    const model = builtinModelMap.get(source) || builtinModelMap.get('suzanne');
    source = model?.id || 'suzanne';
    state.modelSource = source;
    state.animTime = 0;
    state.animFrameIndex = 0;
    state.animLoopIndex = 0;
    state.animSampleTime = 0;
    state.animJitterFrames = 0;
    state.animAccumulator = 0;
    state.animLastMs = performance.now();
    if (model?.kind === 'fbx') {
      resetViewForModel('fbx');
      state.fbxSourceUrl = state.fbxCustomObjectUrl || model.url || '';
      state.fbxSourceLabel = state.fbxCustomObjectUrl ? (state.fbxSourceLabel || 'custom.fbx') : (model.label || 'walking.fbx');
      startFbxMode();
    } else {
      stopFbxMode();
      clearCustomFbxUrl();
      state.fbxSourceUrl = '';
      state.fbxSourceLabel = '';
      resetFbxRuntime();
      resetViewForModel('obj');
      loadBuiltInObjModel(model || builtinModelMap.get('suzanne'));
    }
    syncUi();
    requestSaveSettings();
  }

  async function probeWalkingFbx() {
    try {
      const res = await fetch((builtinModelMap.get('walking')?.url) || FBX_MODEL_URL, { method:'HEAD' });
      state.walkingFbxReady = !!res.ok;
    } catch (_) {
      state.walkingFbxReady = false;
    }
  }

  async function startFbxMode() {
    state.animPlaying = true;
    state.animTime = 0;
    state.animFrameIndex = 0;
    state.animLoopIndex = 0;
    state.animSampleTime = 0;
    state.animJitterFrames = 0;
    state.animAccumulator = 0;
    state.animLastMs = performance.now();
    state.fbxClockMs = state.animLastMs;
    try {
      await ensureFbxRuntime();
      renderFbxFrame(0);
      scheduleRender();
    } catch (err) {
      console.error(err);
    }
  }

  function stopFbxMode() {
    state.animPlaying = false;
  }

  function renderFbxFrame(dt) {
    const rt = state.fbxRuntime;
    if (!rt) return;
    const step = 1 / Math.max(1, state.animFps || 24);
    const duration = rt.duration || 1;
    if (dt > 0) {
      const nextTime = state.animTime + dt;
      if (duration > 0) state.animLoopIndex += Math.max(0, Math.floor(nextTime / duration));
      state.animTime = wrapTime(nextTime, duration);
      state.animFrameIndex += Math.max(1, Math.round(dt / step));
    }
    const sampleTime = computeImpreciseSampleTime(step, duration);
    state.animSampleTime = sampleTime;
    if (rt.mixer) rt.mixer.setTime(sampleTime);
    state.mesh = extractFbxAdapterMesh(rt);
    state.frame = null;
    renderCache.frame = null;
    renderCache.pipelineKey = '';
    invalidateDerivedCaches(renderCache);
    scheduleRender();
  }

  function svgEscape(s){ return String(s).replace(/[&<>"']/g,m=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[m])); }
  function pathD(pts){ if (!pts.length) return ''; return 'M '+pts.map(p=>`${fmt(p.x,1)} ${fmt(p.y,1)}`).join(' L '); }
  function faceD(f){ return `M ${fmt(f.p[0].sx,1)} ${fmt(f.p[0].sy,1)} L ${fmt(f.p[1].sx,1)} ${fmt(f.p[1].sy,1)} L ${fmt(f.p[2].sx,1)} ${fmt(f.p[2].sy,1)} Z`; }
  function buildSvg(frame) {
    const svgKey = buildSvgKey(state, frame);
    if (renderCache.svg.key === svgKey && renderCache.svg.text) return renderCache.svg.text;
    const w=canvas.width,h=canvas.height, parts=[];
    parts.push(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}">`);
    parts.push(`<metadata>${svgEscape(JSON.stringify({ pipeline: 'vectorized-3d-humanized-projection', lineSets: state.lineSets, strokeTools: state.strokeTools, regionSets: state.regionSets, detailPolicy: state.detailPolicyEnabled, vectorBudget: state.vectorBudgetEnabled, selectedFaces: frame.renderSelection?.faceIds?.length ?? frame.workerSelection?.selectedFaces ?? null }))}</metadata>`);
    parts.push(`<rect width="100%" height="100%" fill="${svgEscape(paperColor())}"/>`);
    if (state.paintEnabled) {
      const faces = paintFaces(frame);
      const regions = frame.paintRegions || buildPaintRegions(frame, state);
      parts.push(`<defs><clipPath id="paint-model-mask">`);
      for (const f of faces) parts.push(`<path d="${faceD(f)}"/>`);
      parts.push(`</clipPath></defs>`);
      parts.push(`<g id="paint-regions" clip-path="url(#paint-model-mask)">`);
      for (const region of regions) {
        const blend = region.composite && region.composite !== 'source-over' ? ` style="mix-blend-mode:${region.composite}"` : '';
        const regionSet = region.kind === 'shadow' ? 'shadowRegion' : region.kind === 'highlight' ? 'highlightRegion' : 'baseWash';
        parts.push(`<path data-region-set="${svgEscape(regionSet)}" data-source="${svgEscape(region.kind)}" d="${region.d}" fill="${svgEscape(region.color)}" fill-opacity="${fmt(region.opacity,3)}"${blend}/>`);
      }
      parts.push(`</g>`);
    }
    parts.push(`<g id="shadow-strokes" fill="none" stroke-linecap="round" stroke-linejoin="round">`);
    const inkAlpha = clamp(state.inkDominance || 1, .35, 1.35);
    const inkWidth = lerp(.92, 1.10, clamp01((inkAlpha - .35) / 1));
    let currentShadowLineSet = '';
    for (const m of frame.marks) {
      const lineSetId = m.lineSetId || 'shadowHatch';
      if (lineSetId !== currentShadowLineSet) {
        if (currentShadowLineSet) parts.push(`</g>`);
        currentShadowLineSet = lineSetId;
        parts.push(`<g id="line-set-${svgEscape(lineSetId)}" data-line-set="${svgEscape(lineSetId)}">`);
      }
      const meta = ` data-tool="${svgEscape(m.toolId || '')}" data-line-set="${svgEscape(m.lineSetId || '')}" data-source="${svgEscape(m.sourceType || '')}"`;
      if (m.kind==='dot') parts.push(`<circle${meta} cx="${fmt(m.x,1)}" cy="${fmt(m.y,1)}" r="${fmt(m.r,2)}" fill="${m.color}" fill-opacity="${fmt(clamp01(m.alpha * inkAlpha),3)}"/>`);
      else parts.push(`<path${meta} d="${pathD(m.pts)}" stroke="${m.color}" stroke-opacity="${fmt(clamp01(m.alpha * inkAlpha),3)}" stroke-width="${fmt(m.width * inkWidth,2)}"/>`);
    }
    if (currentShadowLineSet) parts.push(`</g>`);
    parts.push(`</g><g id="contours" fill="none" stroke-linecap="round" stroke-linejoin="round">`);
    let currentContourLineSet = '';
    for (const s of frame.contours) {
      const lineSet = lineSetForContourKind(s.kind, s.visible);
      if (lineSet.enabled === false) continue;
      const len = Math.hypot(s.x2 - s.x1, s.y2 - s.y1);
      if (len < (lineSet.minLengthPx || state.cleanupMinLineLengthPx || 0)) continue;
      const pts = contourVariantPoints(s);
      if (!pts) continue;
      const style = resolveStrokeStyle({
        toolId: lineSet.tool || 'mainInk',
        lineSetId: lineSet.id,
        tone: s.kind === 'contour' ? 1 : (lineSet.strength || .55),
        seed: (s.id + 1) * 19.23 + contourFrameSeed()
      });
      const dash=s.visible?'':' stroke-dasharray="6 5"';
      const op=clamp01(style.alpha * (s.visible ? 1 : .55) * inkAlpha);
      const width=style.width * inkWidth;
      if (lineSet.id !== currentContourLineSet) {
        if (currentContourLineSet) parts.push(`</g>`);
        currentContourLineSet = lineSet.id;
        parts.push(`<g id="line-set-${svgEscape(lineSet.id)}" data-line-set="${svgEscape(lineSet.id)}">`);
      }
      parts.push(`<path data-tier="${s.detailTier ?? 0}" data-tool="${svgEscape(style.toolId || '')}" data-line-set="${svgEscape(lineSet.id || '')}" data-source="${svgEscape(s.kind || '')}" d="${pathD(pts)}" stroke="${style.color}" stroke-opacity="${fmt(op,3)}" stroke-width="${fmt(width,2)}"${dash}/>`);
    }
    if (currentContourLineSet) parts.push(`</g>`);
    parts.push(`</g><metadata>${svgEscape(JSON.stringify({tool:'Susan Shadow Editor v4',method:state.method,flow:state.flowMode,hideOccluded:state.hideOccluded,features:frame.features}))}</metadata></svg>`);
    renderCache.svg.key = svgKey;
    renderCache.svg.text = parts.join('\n');
    return renderCache.svg.text;
  }

  function downloadText(name,mime,text) {
    const blob=new Blob([text],{type:mime});
    const url=URL.createObjectURL(blob);
    const a=document.createElement('a'); a.href=url; a.download=name; document.body.appendChild(a); a.click(); a.remove();
    setTimeout(()=>URL.revokeObjectURL(url),750);
  }
  function exportSvg(){ const frame=state.frame || computeFrame(); downloadText('susan_shadow_editor_v4.svg','image/svg+xml',buildSvg(frame)); }
  function exportPng(){ const a=document.createElement('a'); a.download='susan_shadow_editor_v4.png'; a.href=canvas.toDataURL('image/png'); a.click(); }
  async function exportFbxClip() {
    if (state.modelSource !== 'walking') {
      setModelSource('walking');
    }
    const rt = await ensureFbxRuntime();
    const savedTime = state.animTime;
    const savedSample = state.animSampleTime;
    const buffer = buildFbxClipAmc(rt, {
      fps: 60,
      duration: rt.duration || 1,
    });
    state.animTime = savedTime;
    state.animSampleTime = savedSample;
    if (rt.mixer) rt.mixer.setTime(savedSample || savedTime || 0);
    state.mesh = extractFbxAdapterMesh(rt);
    scheduleRender();
    downloadArrayBuffer('walking.amc', 'application/octet-stream', buffer);
  }
  function exportAtlas() {
    const saved={...state};
    const keys=Object.keys(presets);
    const cols=3, cellW=600, cellH=300;
    const rows=Math.ceil(keys.length/cols);
    const out=document.createElement('canvas'); out.width=cols*cellW; out.height=rows*cellH;
    const o=out.getContext('2d'); o.fillStyle=paperColor(); o.fillRect(0,0,out.width,out.height);
    for (let i=0;i<keys.length;i++) {
      Object.assign(state,presets[keys[i]],{preset:keys[i],auto:false});
      const frame=computeFrame(); renderFrame(frame);
      const x=(i%cols)*cellW, y=Math.floor(i/cols)*cellH;
      o.drawImage(canvas,0,0,canvas.width,canvas.height,x,y,cellW,cellH);
      o.fillStyle='rgba(244,238,227,.88)'; o.fillRect(x+14,y+12,315,34);
      o.fillStyle='#17110b'; o.font='700 18px system-ui'; o.fillText(String(i+1).padStart(2,'0') + ' · ' + keys[i],x+28,y+35);
    }
    Object.assign(state,saved); syncUi(); scheduleRender();
    const a=document.createElement('a'); a.download='susan_shadow_style_atlas_v4.png'; a.href=out.toDataURL('image/png'); a.click();
  }

  function animationLoop(now=performance.now()) {
    resizeCanvas();
    const controlDt = Math.min(.08, Math.max(0, (now - (state.lastTickMs || now)) / 1000));
    state.lastTickMs = now;
    updateCameraFromKeys(controlDt);
    if (state.animPlaying && state.modelSource === 'walking') {
      const dt = Math.min(.12, Math.max(0, (now - (state.animLastMs || now)) / 1000));
      state.animLastMs = now;
      state.animAccumulator += dt;
      const step = 1 / Math.max(1, state.animFps || 24);
      if (state.animAccumulator >= step) {
        const ticks = Math.floor(state.animAccumulator / step);
        state.animAccumulator -= ticks * step;
        renderFbxFrame(ticks * step);
      }
    } else {
      state.animLastMs = now;
    }
    requestAnimationFrame(animationLoop);
  }

  async function init() {
    bindUi();
    loadSavedSettings();
    settingsLoaded = true;
    populateModelSourceOptions();
    syncUi(); resizeCanvas();
    await probeWalkingFbx();
    setModelSource(state.modelSource);
    animationLoop();
  }

  init();
