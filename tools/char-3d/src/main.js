import { state } from './state/defaultState.js';
import { rangeControls, colorControls, checkControls } from './state/controlSchema.js';
import { presets } from './state/stylePresets.js';
import { paintPalettes } from './state/paintPalettes.js';
import { parseOBJ } from './mesh/objParser.js';
import { extractFbxAdapterMesh, ensureFbxRuntime as ensureFbxRuntimeForState, FBX_MODEL_URL } from './mesh/fbxAdapter.js';
import { SUZANNE_OBJ_URL } from './mesh/modelSources.js';
import { TAU, EPS, clamp, clamp01, lerp, deg, fmt, v3, sub, cross, dot, norm, len2, norm2, rot2, mix2, triArea2, hash01, noise, bary2, baryInside, mixPoint, pointFromBary } from './math/core.js';
import { escapeHtml, normalizeHexColor, hexRgb, mixRgb, rgba } from './math/color.js';
import { cameraKeyCodes, isTypingTarget, setModelAngles as setModelAnglesForState, setCameraAngles as setCameraAnglesForState, applyAngleSnap as applyAngleSnapForState, cameraDollyScale as cameraDollyScaleForState, updateCameraFromKeys as updateCameraFromKeysForState } from './app/cameraControls.js';
import { computeImpreciseSampleTime as computeImpreciseSampleTimeForState, randomnessFrameSeed as randomnessFrameSeedForState, shadowRandomSeed as shadowRandomSeedForState } from './npr/randomSeeds.js';
import { buildPaintRegions } from './paint/paintRegions.js';
import { createPerfStats, resetPerfFrame, markCacheHit, markCacheMiss, timeSection, timeSectionEnd, finishPerfFrame, formatPerfStats } from './render/perfStats.js';
import {
  createRenderCache,
  buildPipelineKey,
  buildBackgroundLayerKey,
  buildPaintLayerKey,
  buildSvgKey,
  ensureLayerCanvas,
  getReusableDepthBuffers,
  invalidateDerivedCaches
} from './render/renderCache.js';
import { DIRTY_FLAGS, createDirtyFlags, markDirty, clearDirty } from './render/dirtyFlags.js';
'use strict';

  const canvas = document.getElementById('view');
  const ctx = canvas.getContext('2d', { alpha: false });
  const statusEl = document.getElementById('status');
  const legendEl = document.getElementById('legend');

  const $ = id => document.getElementById(id);

  const STORAGE_KEY = 'char3d.strokes.settings.v1';
  const persistedFields = [
    ...Object.keys(rangeControls),
    ...colorControls,
    ...checkControls,
    'controlMode',
    'angleSnap',
    'method',
    'flowMode',
    'preset',
    'paintBrush',
    'paintPalette',
    'modelSource',
    'animFps',
    'mode',
    'rawYaw',
    'rawPitch',
    'rawCameraYaw',
    'rawCameraPitch'
  ];
  let settingsLoaded = false;
  let saveSettingsTimer = 0;
  let meshRevision = 0;
  const renderCache = createRenderCache();
  const perfStats = createPerfStats();
  const dirtyFlags = createDirtyFlags();
  const displayOnlyKeys = new Set(['paintEnabled','faceWash','contours','tone','flow','depthDebug','seedDebug','sortFaces','inkDominance']);
  const visibilityKeys = new Set(['hideOccluded','backface','depthClipStrokes','clipToFaces','showHidden','depthEps','creases','suggestive','contactLines']);
  const nprKeys = new Set(['method','mode','flowMode','density','layers','threshold','strokeLen','spacing','strokeWidth','curvature','crossAngle','dotSize','wobble','jitter','strokeCrookedness','strokeKinkChance','strokeToneRamp','shadowFrameDrift','shadowLoopRedraw','shadowLayoutJitter','spacingVar','lengthVar','widthVar','taper','breakup','overdraw','contourHumanize','contourDrift','contourWobble','contourGaps','contourFrameVariance','shadowsEnabled']);

  function renderScopeForKey(key) {
    if (String(key).startsWith('paint')) return DIRTY_FLAGS.PAINT;
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
      scheduleRender();
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

  function setModelAngles(yaw, pitch) {
    setModelAnglesForState(state, yaw, pitch);
  }

  function setCameraAngles(yaw, pitch) {
    setCameraAnglesForState(state, yaw, pitch);
  }

  function applyAngleSnap() {
    applyAngleSnapForState(state);
  }

  function cameraDollyScale() {
    return cameraDollyScaleForState(state);
  }

  function transformFrame() {
    const mesh = state.mesh;
    const yaw = deg(state.yaw), pitch = deg(state.pitch);
    const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
    const cameraYaw = deg(-state.cameraYaw), cameraPitch = deg(-state.cameraPitch);
    const ccy = Math.cos(cameraYaw), csy = Math.sin(cameraYaw);
    const ccp = Math.cos(cameraPitch), csp = Math.sin(cameraPitch);
    const fbxAdapter = mesh?.sourceType === 'fbx';
    const scale = Math.min(canvas.width, canvas.height) * 0.36 * state.zoom * cameraDollyScale() * (fbxAdapter ? .78 : 1);
    const centerX = canvas.width/2 + (fbxAdapter ? Math.min(180, canvas.width * .13) : 0);
    const centerY = canvas.height/2 - (fbxAdapter ? Math.min(42, canvas.height * .04) : 0);
    const verts = mesh.verts.map((p, vi) => {
      const x1 = p.x*cy + p.z*sy;
      const z1 = -p.x*sy + p.z*cy;
      const y2 = p.y*cp - z1*sp;
      const z2 = p.y*sp + z1*cp;
      const vx = x1 - state.cameraX;
      const vy = y2 - state.cameraY;
      const vz = z2 - state.cameraZ;
      const x3 = vx*ccy + vz*csy;
      const z3 = -vx*csy + vz*ccy;
      const y4 = vy*ccp - z3*csp;
      const z4 = vy*csp + z3*ccp;
      let screenX = centerX + x3*scale;
      let screenY = centerY - y4*scale;
      if (state.projectionWobble > 0) {
        const seed = (vi + 1) * 409.17 + randomnessFrameSeed() * 23.91;
        const amp = state.projectionWobble * (fbxAdapter ? 1.18 : 1);
        screenX += noise(seed, 1) * amp;
        screenY += noise(seed, 2) * amp;
      }
      return {x:x3,y:y4,z:z4,sx:screenX,sy:screenY};
    });
    const L = lightVector();
    const faces = mesh.faces.map((face, id) => {
      const a=verts[face.v[0]], b=verts[face.v[1]], c=verts[face.v[2]];
      const n = norm(cross(sub(b,a), sub(c,a)));
      const cx=(a.sx+b.sx+c.sx)/3, cy2=(a.sy+b.sy+c.sy)/3;
      const depth=(a.z+b.z+c.z)/3;
      const area=triArea2(a,b,c);
      const ndotl=dot(n,L);
      const shade = 1 - clamp01(ndotl * .5 + .5);
      const rim = 1 - Math.abs(n.z);
      const contact = contactScore(cy2, n);
      let tone = clamp01(shade * .86 + rim * state.edgeDark * .36 + contact * state.contact * .42);
      tone = Math.pow(tone, lerp(1.55, .58, clamp01(state.core/2)));
      if (state.simplify > 0.01) {
        const bands = Math.round(lerp(10, 3, state.simplify));
        tone = Math.round(tone * bands) / bands;
      }
      return {id, v:face.v, p:[a,b,c], n, area, cx, cy:cy2, depth, front:n.z>0, tone, ndotl, flow:{x:1,y:0}, visible:false, visibility:0};
    });
    return {verts, faces, L, db:null, contours:[], marks:[]};
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
    for (const f of frame.faces) {
      if (state.backface && !f.front) continue;
      rasterTri(f, depth, owner, w, h, sx, sy);
    }
    return {w,h,depth,owner,sx,sy,quality};
  }

  function rasterTri(f, depth, owner, w, h, sx, sy) {
    const p=f.p;
    const ax=p[0].sx*sx, ay=p[0].sy*sy, az=p[0].z;
    const bx=p[1].sx*sx, by=p[1].sy*sy, bz=p[1].z;
    const cx=p[2].sx*sx, cy=p[2].sy*sy, cz=p[2].z;
    const minX=clamp(Math.floor(Math.min(ax,bx,cx))-1,0,w-1), maxX=clamp(Math.ceil(Math.max(ax,bx,cx))+1,0,w-1);
    const minY=clamp(Math.floor(Math.min(ay,by,cy))-1,0,h-1), maxY=clamp(Math.ceil(Math.max(ay,by,cy))+1,0,h-1);
    const den = (by-cy)*(ax-cx) + (cx-bx)*(ay-cy);
    if (Math.abs(den) < EPS) return;
    for (let y=minY;y<=maxY;y++) for (let x=minX;x<=maxX;x++) {
      const px=x+.5, py=y+.5;
      const u=((by-cy)*(px-cx)+(cx-bx)*(py-cy))/den;
      const v=((cy-ay)*(px-cx)+(ax-cx)*(py-cy))/den;
      const ww=1-u-v;
      if (u < -0.005 || v < -0.005 || ww < -0.005) continue;
      const z = u*az + v*bz + ww*cz;
      const idx=y*w+x;
      if (z > depth[idx]) { depth[idx]=z; owner[idx]=f.id; }
    }
  }

  function sampleDepth(db, x, y) {
    if (!db || x<0 || y<0 || x>=canvas.width || y>=canvas.height) return -1e9;
    const ix=clamp(Math.floor(x*db.sx),0,db.w-1), iy=clamp(Math.floor(y*db.sy),0,db.h-1);
    return db.depth[iy*db.w+ix];
  }

  function isVisiblePoint(db, x, y, z) {
    if (!state.hideOccluded || !state.depthClipStrokes) return x>=0 && y>=0 && x<canvas.width && y<canvas.height;
    return z >= sampleDepth(db, x, y) - state.depthEps;
  }

  function computeVisibilityAndFlow(frame) {
    const db = frame.db;
    for (const f of frame.faces) {
      const p=f.p;
      if (state.backface && !f.front) { f.visible=false; f.visibility=0; continue; }
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
      f.flow = computeFlow(f, frame);
    }
  }

  function computeFlow(f, frame) {
    const p=f.p;
    let best = {x:p[1].sx-p[0].sx, y:p[1].sy-p[0].sy};
    let bestLen = len2(best);
    for (let i=0;i<3;i++) {
      const a=p[i], b=p[(i+1)%3];
      const e={x:b.sx-a.sx, y:b.sy-a.sy};
      const l=len2(e);
      if (l > bestLen) { best=e; bestLen=l; }
    }
    const form = norm2(best);
    const radial = norm2({x:f.cx-canvas.width/2, y:f.cy-canvas.height/2});
    const crossContour = norm2({x:-radial.y, y:radial.x});
    const light = norm2({x:frame.L.x, y:-frame.L.y});
    const terminator = norm2({x:-light.y, y:light.x});
    const parallel = norm2(rot2({x:1,y:0}, deg(-22)));
    switch (state.flowMode) {
      case 'parallel': return parallel;
      case 'form': return form;
      case 'crossContour': return mix2(crossContour, form, .18);
      case 'silhouette': return crossContour;
      case 'light': return light;
      case 'terminator': return terminator;
      default: return norm2({x:form.x*.50 + crossContour.x*.32 + terminator.x*.20, y:form.y*.50 + crossContour.y*.32 + terminator.y*.20});
    }
  }

  function computeContours(frame) {
    const out=[];
    const mesh=state.mesh;
    const fbxAdapter = mesh?.sourceType === 'fbx';
    for (const e of mesh.edges) {
      const f0=frame.faces[e.faces[0]];
      const f1=e.faces.length>1 ? frame.faces[e.faces[1]] : null;
      const boundary=!f1;
      const silhouette=f1 ? (f0.front !== f1.front) : true;
      const crease=f1 ? dot(f0.n, f1.n) < .70 : false;
      const toneBreak=f1 ? Math.abs(f0.tone - f1.tone) > .32 : false;
      const a=frame.verts[e.a], b=frame.verts[e.b];
      const screenLen = Math.hypot(a.sx-b.sx, a.sy-b.sy);
      let kind='';
      if (boundary || silhouette) kind='contour';
      else if (!fbxAdapter && state.creases && crease) kind='crease';
      else if (!fbxAdapter && state.suggestive && toneBreak) kind='suggestive';
      if (!kind) continue;
      if (fbxAdapter && screenLen < 2.4) continue;
      const mx=(a.sx+b.sx)/2, my=(a.sy+b.sy)/2, mz=(a.z+b.z)/2;
      const visible = isVisiblePoint(frame.db, mx, my, mz);
      if (!visible && !state.showHidden) continue;
      out.push({x1:a.sx,y1:a.sy,z1:a.z,x2:b.sx,y2:b.sy,z2:b.z,kind,visible,id:out.length,edgeKey:`${e.a}_${e.b}`});
    }
    return out;
  }

  function generateMarks(frame) {
    const marks=[];
    const fbxAdapter = state.mesh?.sourceType === 'fbx';
    const minArea = fbxAdapter ? 0.22 : 1.5;
    const faces = frame.faces.filter(f => f.area > minArea && f.visible && (!state.backface || f.front));
    faces.sort((a,b)=>a.depth-b.depth);
    const baseMarks = fbxAdapter ? 760 : 450;
    const markRange = fbxAdapter ? 2600 : 1800;
    const maxMarks = Math.floor(baseMarks + markRange * clamp01(state.density/2) * lerp(1.15,.45,state.economy));
    let used=0;
    for (const f of faces) {
      if (used >= maxMarks) break;
      const made = generateFaceMarks(f, frame, marks, maxMarks-used);
      used += made;
    }
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

    // Stable per-face seed must be initialized before any hash/noise call.
    // v2 used `seed` before this declaration, which caused a temporal dead-zone
    // ReferenceError and killed every render frame.
    const shadowSeed = shadowRandomSeed();
    const seed=(f.id+1)*1009.133 + shadowSeed * (1 + hash01(f.id + 19.3) * .7);

    let n = Math.floor(raw);
    if (hash01(seed + 991.7) < raw - n) n++;
    if (tone > .08 && raw > .045 && hash01(seed + 113.9) < raw * 1.8) n = Math.max(n, 1);
    n = clamp(n, 0, Math.min(42, budget));
    let made=0;
    for (let i=0; i<n && made<budget; i++) {
      const b = stableBary(seed, i, state.spacingVar);
      const c = pointFromBary(f, b.u, b.v, b.w);
      c.x += noise(seed, i+10) * state.jitter * spacing * .35;
      c.y += noise(seed, i+20) * state.jitter * spacing * .35;
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

  function toolStyle(tone) {
    let color='#17110b', alpha=.88, width=state.strokeWidth;
    if (state.mode === 'PENCIL') { color='#2a2621'; alpha=.34; width*=.72; }
    if (state.mode === 'BRUSH') { color='#17110b'; alpha=.68; width*=1.75; }
    return {color, alpha:alpha*lerp(.48,1,tone), width};
  }

  function addMark(out, f, frame, c, tone, seed) {
    const style=toolStyle(tone);
    const method=state.method;
    if (method === 'stipple') { addDot(out,c,tone,seed,style,false); return; }
    if (method === 'halftone') { addDot(out,c,tone,seed,style,true); return; }
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

  function addDot(out,c,tone,seed,style,halftone) {
    let r = state.dotSize * (halftone ? lerp(.55,2.0,tone) : lerp(.55,1.22,tone));
    r *= 1 + noise(seed,2) * (halftone ? .10 : .45) * state.jitter;
    out.push({kind:'dot',x:c.x,y:c.y,z:c.z,r:Math.max(.25,r),color:style.color,alpha:style.alpha});
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
      out.push({kind:'line',pts:seg,color:style.color,alpha:style.alpha,width,taper:clamp01(state.taper*(opt.taperMul||1)),dry:!!opt.dry,seed,inkRamp,rampDir});
    }
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
    const frame=timeSection(perfStats, 'projection', () => transformFrame());
    frame.db=timeSection(perfStats, 'depth', () => buildDepthBuffer(frame));
    timeSection(perfStats, 'visibility', () => computeVisibilityAndFlow(frame));
    frame.contours=timeSection(perfStats, 'contours', () => computeContours(frame));
    frame.marks=state.shadowsEnabled ? timeSection(perfStats, 'marks', () => generateMarks(frame)) : [];
    renderCache.pipelineKey = pipelineKey;
    renderCache.frame = frame;
    return frame;
  }

  function renderFrame(frame) {
    ctx.save();
    ctx.setTransform(1,0,0,1,0,0);
    drawCachedBackground();
    drawCachedPaintLayer(frame);
    if (state.depthDebug) drawDepthDebug(frame.db);
    if (state.shadowsEnabled) drawMarks(frame.marks);
    if (state.contours) drawContours(frame.contours);
    if (state.flow) drawFlow(frame);
    if (state.seedDebug) drawSeeds(frame.marks);
    ctx.restore();
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
    const faces = state.sortFaces ? frame.faces.slice().sort((a,b)=>a.depth-b.depth) : frame.faces.slice();
    frame.paintFaces = faces.filter(f => f.visible && (!state.backface || f.front));
    frame.paintFacesKey = key;
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
    const buckets = new Map();
    for (const f of faces) {
      const tone = clamp01(options.tone ? options.tone(f) : f.tone);
      if (options.skip && options.skip(f, tone)) continue;
      const toneBin = clamp(Math.round(tone * toneBins), 0, toneBins);
      const visibilityBin = clamp(Math.round(clamp01(f.visibility) * visibilityBins), 0, visibilityBins);
      const key = `${toneBin}:${visibilityBin}`;
      let bucket = buckets.get(key);
      if (!bucket) {
        bucket = { path: new Path2D(), count: 0, toneSum: 0, visibilitySum: 0 };
        buckets.set(key, bucket);
      }
      addFaceToPath(bucket.path, f);
      bucket.count++;
      bucket.toneSum += tone;
      bucket.visibilitySum += clamp01(f.visibility);
    }
    for (const bucket of buckets.values()) {
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
      targetCtx.fill(new Path2D(region.d));
      return;
    }
    regionPath(targetCtx, region);
    targetCtx.fill();
  }

  function clipPaintRegion(targetCtx, region) {
    if (typeof Path2D !== 'undefined' && region.d) {
      targetCtx.clip(new Path2D(region.d));
      return;
    }
    regionPath(targetCtx, region);
    targetCtx.clip();
  }

  function drawPaintRegions(regions, targetCtx) {
    for (const region of regions) {
      targetCtx.save();
      targetCtx.globalCompositeOperation = region.composite || 'source-over';
      targetCtx.globalAlpha = clamp01(region.opacity);
      if (region.blur > .05) targetCtx.filter = `blur(${fmt(region.blur,2)}px)`;
      targetCtx.fillStyle = region.color;
      fillPaintRegion(targetCtx, region);
      targetCtx.restore();
    }
  }

  function clipProjectedPaintMask(targetCtx, faces) {
    targetCtx.beginPath();
    for (const f of faces) {
      targetCtx.moveTo(f.p[0].sx, f.p[0].sy);
      targetCtx.lineTo(f.p[1].sx, f.p[1].sy);
      targetCtx.lineTo(f.p[2].sx, f.p[2].sy);
      targetCtx.closePath();
    }
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
    targetCtx.save();
    clipProjectedPaintMask(targetCtx, faces);
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
    for (const m of marks) {
      if (m.kind === 'dot') {
        ctx.globalAlpha=clamp01(m.alpha * inkAlpha); ctx.fillStyle=m.color; ctx.beginPath(); ctx.arc(m.x,m.y,m.r,0,TAU); ctx.fill();
      } else if (m.pts.length > 1) {
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
    if (state.modelSource === 'walking') return Math.floor(state.animFrameIndex * clamp01(state.contourFrameVariance));
    return Math.floor((state.rawYaw ?? state.yaw) * .18 * clamp01(state.contourFrameVariance));
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

  function drawContours(segs) {
    ctx.save(); ctx.lineCap='round'; ctx.lineJoin='round';
    const inkAlpha = clamp(state.inkDominance || 1, .35, 1.35);
    const inkWidth = lerp(.92, 1.12, clamp01((inkAlpha - .35) / 1));
    for (const s of segs) {
      const pts = contourVariantPoints(s);
      if (!pts) continue;
      ctx.globalAlpha=clamp01((s.visible ? (s.kind==='contour'? .92:.46) : .24) * inkAlpha);
      ctx.strokeStyle=s.visible ? '#17110b' : '#65584d';
      const widthNoise = state.contourHumanize && s.kind==='contour' ? lerp(.88,1.18,hash01((s.id+1)*19.23 + contourFrameSeed())) : 1;
      ctx.lineWidth=(s.visible ? (s.kind==='contour'?1.45:.76) : .85) * widthNoise * inkWidth;
      ctx.setLineDash(s.visible ? [] : [6,5]);
      strokeSmooth(pts);
    }
    ctx.restore();
  }

  function drawFlow(frame) {
    ctx.save(); ctx.strokeStyle='rgba(20,94,103,.68)'; ctx.lineWidth=1; ctx.lineCap='round';
    let k=0;
    for (const f of frame.faces) {
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

  function drawDepthDebug(db) {
    const img=ctx.createImageData(db.w,db.h); let min=1e9,max=-1e9;
    for (const z of db.depth) if (z>-1e8) { min=Math.min(min,z); max=Math.max(max,z); }
    const span=Math.max(.001,max-min);
    for (let i=0;i<db.depth.length;i++) {
      const z=db.depth[i], alive=z>-1e8, t=alive?(z-min)/span:0;
      img.data[i*4]=20; img.data[i*4+1]=Math.floor(80+t*130); img.data[i*4+2]=Math.floor(100+t*130); img.data[i*4+3]=alive?110:0;
    }
    const tmp=document.createElement('canvas'); tmp.width=db.w; tmp.height=db.h; tmp.getContext('2d').putImageData(img,0,0);
    ctx.drawImage(tmp,0,0,canvas.width,canvas.height);
  }

  function updateStatus(frame) {
    const visible=frame.faces.filter(f=>f.visible && (!state.backface || f.front)).length;
    const model = state.mesh?.name || 'model';
    const tween = state.impreciseTween ? ` · drift: <b>${fmt(state.animJitterFrames,2)}f</b>` : '';
    const topology = state.mesh?.sourceType === 'fbx' ? ` · topology: <b>cached</b> · vtx frame: <b>${state.mesh.frameVersion || 0}</b>` : '';
    const anim = state.modelSource === 'walking' ? ` · FBX adapter: <b>${state.animFps} fps</b>${tween} · source: <b>${state.walkingFbxReady?'walking.fbx':'not found'}</b>${topology}` : '';
    const html = `model: <b>${escapeHtml(model)}</b>${anim}<br>` +
      `control: <b>${state.controlMode}</b> · camera xyz: ${fmt(state.cameraX,2)}, ${fmt(state.cameraY,2)}, ${fmt(state.cameraZ,2)} · look: ${Math.round(state.cameraYaw)}°/${Math.round(state.cameraPitch)}°<br>` +
      `faces: ${frame.faces.length} · visible faces: ${visible} · strokes/dots: ${frame.marks.length} · contours: ${frame.contours.length}<br>` +
      `method: <b>${state.method}</b> · flow: <b>${state.flowMode}</b> · dirty: <b>${dirtyFlags.last}</b> · hide occluded: <b>${state.hideOccluded?'ON':'OFF'}</b> · depth buffer: ${frame.db.w}×${frame.db.h}<br>` +
      formatPerfStats(perfStats, fmt);
    if (statusEl) statusEl.innerHTML = html;
    if (legendEl) legendEl.innerHTML = state.hideOccluded ? 'Depth visibility ON: tylne ucho i tylne stroke’i są odcinane przez bufor głębi.' : 'Depth visibility OFF: tryb diagnostyczny pokazuje także zasłonięte regiony.';
  }

  function scheduleRender(scope = DIRTY_FLAGS.PROJECTION) {
    markDirty(dirtyFlags, scope);
    state.needsRender = true;
    if (state.scheduled) return;
    state.scheduled = true;
    requestAnimationFrame(() => {
      state.scheduled = false;
      if (!state.mesh || !state.needsRender) return;
      state.needsRender = false;
      try {
        const totalStart = performance.now();
        resetPerfFrame(perfStats, false);
        const frame = computeFrame();
        state.frame = frame;
        const drawStart = performance.now();
        if (dirtyFlags.projection || dirtyFlags.visibility || dirtyFlags.paint) renderCache.paint.key = '';
        if (dirtyFlags.mesh || dirtyFlags.projection || dirtyFlags.visibility || dirtyFlags.npr || dirtyFlags.paint) {
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
        if (statusEl) statusEl.innerHTML = '<b>Render error:</b> ' + escapeHtml(String(err && err.message ? err.message : err));
      }
    });
  }

  function updateCameraFromKeys(dt) {
    if (!updateCameraFromKeysForState(state, dt)) return false;
    syncUi();
    scheduleRender();
    requestSaveSettings();
    return true;
  }

  function snapshotSettings() {
    const values = {};
    for (const key of persistedFields) values[key] = state[key];
    return { version: 1, savedAt: Date.now(), values };
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
        if (key in rangeControls || ['angleSnap','animFps','rawYaw','rawPitch','rawCameraYaw','rawCameraPitch'].includes(key)) {
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
      state.paintPalette = paintPalettes[state.paintPalette] ? state.paintPalette : 'cleanComic';
      state.paintBrush = ['watercolor','gouache','comicCel','inkWash'].includes(state.paintBrush) ? state.paintBrush : 'watercolor';
      state.modelSource = state.modelSource === 'walking' ? 'walking' : 'suzanne';
      state.animFps = [2,4,8,12,24,30,60].includes(Number(state.animFps)) ? Number(state.animFps) : 24;
      state.rawYaw = Number.isFinite(Number(state.rawYaw)) ? Number(state.rawYaw) : state.yaw;
      state.rawPitch = Number.isFinite(Number(state.rawPitch)) ? Number(state.rawPitch) : state.pitch;
      state.rawCameraYaw = Number.isFinite(Number(state.rawCameraYaw)) ? Number(state.rawCameraYaw) : state.cameraYaw;
      state.rawCameraPitch = Number.isFinite(Number(state.rawCameraPitch)) ? Number(state.rawCameraPitch) : state.cameraPitch;
      applyAngleSnap();
      return true;
    } catch (err) {
      console.warn('Settings load failed', err);
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
    if ($('controlModeV')) $('controlModeV').textContent=state.controlMode === 'freelook' ? 'free' : 'orbit';
    if ($('angleSnap')) $('angleSnap').value=String(state.angleSnap);
    if ($('angleSnapV')) $('angleSnapV').textContent=state.angleSnap ? `${state.angleSnap}°` : 'smooth';
    for (const id of checkControls) { const el=$(id); if (el) el.checked=!!state[id]; }
    if ($('method')) $('method').value=state.method;
    if ($('flowMode')) $('flowMode').value=state.flowMode;
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
        if (key === 'yaw') setModelAngles(value, state.rawPitch ?? state.pitch);
        else if (key === 'pitch') setModelAngles(state.rawYaw ?? state.yaw, value);
        else if (key === 'cameraYaw') setCameraAngles(value, state.rawCameraPitch ?? state.cameraPitch);
        else if (key === 'cameraPitch') setCameraAngles(state.rawCameraYaw ?? state.cameraYaw, value);
        else state[key] = value;
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
    $('preset')?.addEventListener('change', e => applyPreset(e.target.value));
    $('paintBrush')?.addEventListener('change', e => { state.paintBrush=e.target.value; syncUi(); scheduleRender(DIRTY_FLAGS.PAINT); requestSaveSettings(); });
    $('paintPalette')?.addEventListener('change', e => applyPaintPalette(e.target.value));
    $('controlMode')?.addEventListener('change', e => { state.controlMode=e.target.value; state.auto=false; syncUi(); scheduleRender(); requestSaveSettings(); });
    $('angleSnap')?.addEventListener('change', e => { state.angleSnap=Number(e.target.value)||0; applyAngleSnap(); syncUi(); scheduleRender(); requestSaveSettings(); });
    $('modelSource')?.addEventListener('change', e => setModelSource(e.target.value));
    $('animFps')?.addEventListener('change', e => { state.animFps=Number(e.target.value)||24; state.animAccumulator=0; requestSaveSettings(); });
    for (const b of document.querySelectorAll('.modeButtons button')) b.addEventListener('click', () => { state.mode=b.dataset.mode; syncUi(); scheduleRender(DIRTY_FLAGS.NPR); requestSaveSettings(); });
    for (const b of document.querySelectorAll('.tabBtn')) b.addEventListener('click', () => {
      document.querySelectorAll('.tabBtn').forEach(x=>x.classList.toggle('active', x===b));
      document.querySelectorAll('.tabPane').forEach(p=>p.classList.toggle('active', p.id===b.dataset.tab));
    });
    $('reset')?.addEventListener('click', () => { Object.assign(state,{controlMode:'orbit',angleSnap:0,yaw:-24,pitch:12,zoom:1,cameraYaw:0,cameraPitch:0,cameraX:0,cameraY:0,cameraZ:0,rawYaw:-24,rawPitch:12,rawCameraYaw:0,rawCameraPitch:0,lightAz:-42,lightEl:42,auto:true}); state.pressedKeys.clear(); syncUi(); scheduleRender(); requestSaveSettings(); });
    $('exportSvg')?.addEventListener('click', exportSvg);
    $('exportPng')?.addEventListener('click', exportPng);
    $('exportAtlas')?.addEventListener('click', exportAtlas);
    $('file')?.addEventListener('change', e => loadFile(e.target.files && e.target.files[0]));
    canvas.addEventListener('contextmenu', e => e.preventDefault());
    canvas.addEventListener('pointerdown', e => {
      state.dragging=true;
      state.lastX=e.clientX;
      state.lastY=e.clientY;
      state.auto=false;
      state.dragMode = (e.button === 1 || e.altKey) ? 'pan' : ((state.controlMode === 'freelook' || e.button === 2 || e.shiftKey) ? 'camera' : 'model');
      canvas.setPointerCapture?.(e.pointerId);
      syncUi();
    });
    canvas.addEventListener('pointermove', e => {
      if (!state.dragging) return;
      const dx=e.clientX-state.lastX, dy=e.clientY-state.lastY;
      state.lastX=e.clientX; state.lastY=e.clientY;
      if (state.dragMode === 'camera') {
        state.controlMode='freelook';
        setCameraAngles((state.rawCameraYaw ?? state.cameraYaw)+dx*.28, (state.rawCameraPitch ?? state.cameraPitch)+dy*.24);
      } else if (state.dragMode === 'pan') {
        const pan = 1 / (Math.min(canvas.width, canvas.height) * .36 * state.zoom * cameraDollyScale());
        state.cameraX=clamp(state.cameraX-dx*pan,-3,3);
        state.cameraY=clamp(state.cameraY+dy*pan,-3,3);
      } else {
        setModelAngles((state.rawYaw ?? state.yaw)+dx*.45, (state.rawPitch ?? state.pitch)+dy*.35);
      }
      syncUi();
      scheduleRender();
      requestSaveSettings();
    });
    canvas.addEventListener('pointerup', e => { state.dragging=false; try{canvas.releasePointerCapture?.(e.pointerId);}catch(_){} });
    canvas.addEventListener('pointercancel', e => { state.dragging=false; try{canvas.releasePointerCapture?.(e.pointerId);}catch(_){} });
    canvas.addEventListener('wheel', e => { e.preventDefault(); state.zoom=clamp(state.zoom*Math.exp(-e.deltaY*.001),.55,1.8); syncUi(); scheduleRender(); requestSaveSettings(); }, {passive:false});
    window.addEventListener('resize', () => { resizeCanvas(); scheduleRender(); });
    window.addEventListener('keydown', e => {
      if (isTypingTarget(e.target)) return;
      if (cameraKeyCodes.has(e.code)) {
        e.preventDefault();
        state.pressedKeys.add(e.code);
        if (!e.shiftKey) state.controlMode='freelook';
        state.auto=false;
        syncUi();
        return;
      }
      if (e.code==='Space') { e.preventDefault(); state.auto=!state.auto; syncUi(); scheduleRender(); requestSaveSettings(); }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase()==='e') { e.preventDefault(); exportSvg(); }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase()==='p') { e.preventDefault(); exportPng(); }
      if (e.key==='1') { state.method='hatching'; syncUi(); scheduleRender(); requestSaveSettings(); }
      if (e.key==='2') { state.method='crosshatch'; syncUi(); scheduleRender(); requestSaveSettings(); }
      if (e.key==='3') { state.method='stipple'; syncUi(); scheduleRender(); requestSaveSettings(); }
      if (e.key==='4') { state.method='drybrush'; syncUi(); scheduleRender(); requestSaveSettings(); }
    });
    window.addEventListener('keyup', e => { state.pressedKeys.delete(e.code); });
    window.addEventListener('blur', () => state.pressedKeys.clear());
    window.addEventListener('dragover', e => e.preventDefault());
    window.addEventListener('drop', e => { e.preventDefault(); loadFile(e.dataTransfer.files && e.dataTransfer.files[0]); });
  }

  function applyPreset(key) {
    const p=presets[key]; if (!p) return;
    Object.assign(state,p);
    state.preset=key;
    state.rawYaw = state.yaw;
    state.rawPitch = state.pitch;
    state.rawCameraYaw = state.cameraYaw;
    state.rawCameraPitch = state.cameraPitch;
    applyAngleSnap();
    syncUi();
    scheduleRender();
    requestSaveSettings();
  }

  function loadFile(file) {
    if (!file) return;
    const reader=new FileReader();
    reader.onload=()=>loadOBJText(String(reader.result || ''), file.name || 'dropped OBJ');
    reader.readAsText(file);
  }

  function loadOBJText(text, name) {
    try {
      const mesh = parseOBJ(text);
      mesh.name = name;
      mesh.cacheId = `obj:${++meshRevision}:${name}:${text.length}`;
      mesh.frameVersion = 0;
      state.mesh = mesh;
      state.frame = null;
      renderCache.frame = null;
      renderCache.pipelineKey = '';
      invalidateDerivedCaches(renderCache);
      scheduleRender(DIRTY_FLAGS.MESH);
    }
    catch(err) { console.error(err); if (statusEl) statusEl.textContent='OBJ error: '+err.message; }
  }


  async function loadSuzanneModel() {
    try {
      const res = await fetch(SUZANNE_OBJ_URL);
      if (!res.ok) throw new Error(`Suzanne OBJ fetch failed: ${res.status}`);
      const obj = await res.text();
      loadOBJText(obj, 'embedded Suzanne/Susan OBJ');
    } catch (err) {
      console.error(err);
      if (statusEl) statusEl.textContent = 'OBJ error: ' + (err && err.message ? err.message : err);
    }
  }

  async function ensureFbxRuntime() {
    return ensureFbxRuntimeForState(state);
  }
  function setModelSource(source) {
    state.modelSource = source;
    state.animTime = 0;
    state.animFrameIndex = 0;
    state.animLoopIndex = 0;
    state.animSampleTime = 0;
    state.animJitterFrames = 0;
    state.animAccumulator = 0;
    state.animLastMs = performance.now();
    if (source === 'walking') {
      startFbxMode();
    } else {
      stopFbxMode();
      loadSuzanneModel();
    }
    syncUi();
    requestSaveSettings();
  }

  async function probeWalkingFbx() {
    try {
      const res = await fetch(FBX_MODEL_URL, { method:'HEAD' });
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
    if (statusEl) statusEl.innerHTML = 'loading <b>walking.fbx</b>...';
    try {
      await ensureFbxRuntime();
      renderFbxFrame(0);
      scheduleRender();
    } catch (err) {
      console.error(err);
      if (statusEl) statusEl.innerHTML = '<b>FBX error:</b> ' + escapeHtml(state.fbxError || err.message || err);
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
        parts.push(`<path d="${region.d}" fill="${svgEscape(region.color)}" fill-opacity="${fmt(region.opacity,3)}"${blend}/>`); 
      }
      parts.push(`</g>`);
    }
    parts.push(`<g id="shadow-strokes" fill="none" stroke-linecap="round" stroke-linejoin="round">`);
    const inkAlpha = clamp(state.inkDominance || 1, .35, 1.35);
    const inkWidth = lerp(.92, 1.10, clamp01((inkAlpha - .35) / 1));
    for (const m of frame.marks) {
      if (m.kind==='dot') parts.push(`<circle cx="${fmt(m.x,1)}" cy="${fmt(m.y,1)}" r="${fmt(m.r,2)}" fill="${m.color}" fill-opacity="${fmt(clamp01(m.alpha * inkAlpha),3)}"/>`);
      else parts.push(`<path d="${pathD(m.pts)}" stroke="${m.color}" stroke-opacity="${fmt(clamp01(m.alpha * inkAlpha),3)}" stroke-width="${fmt(m.width * inkWidth,2)}"/>`);
    }
    parts.push(`</g><g id="contours" fill="none" stroke-linecap="round" stroke-linejoin="round">`);
    for (const s of frame.contours) {
      const pts = contourVariantPoints(s);
      if (!pts) continue;
      const dash=s.visible?'':' stroke-dasharray="6 5"';
      const op=clamp01((s.visible?(s.kind==='contour'?.92:.46):.24) * inkAlpha);
      const width=(s.visible?(s.kind==='contour'?1.45:.76):.85) * inkWidth;
      parts.push(`<path d="${pathD(pts)}" stroke="#17110b" stroke-opacity="${fmt(op,3)}" stroke-width="${fmt(width,2)}"${dash}/>`);
    }
    parts.push(`</g><metadata>${svgEscape(JSON.stringify({tool:'Susan Shadow Editor v4',method:state.method,flow:state.flowMode,hideOccluded:state.hideOccluded}))}</metadata></svg>`);
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
  function exportAtlas() {
    const saved={...state};
    const keys=Object.keys(presets);
    const cols=3, cellW=600, cellH=300;
    const rows=Math.ceil(keys.length/cols);
    const out=document.createElement('canvas'); out.width=cols*cellW; out.height=rows*cellH;
    const o=out.getContext('2d'); o.fillStyle=paperColor(); o.fillRect(0,0,out.width,out.height);
    for (let i=0;i<keys.length;i++) {
      Object.assign(state,presets[keys[i]],{preset:keys[i],auto:false,yaw:-26+(i%3)*10,pitch:12});
      state.rawYaw=state.yaw; state.rawPitch=state.pitch; applyAngleSnap();
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
    if (state.auto && !state.dragging) {
      setModelAngles((state.rawYaw ?? state.yaw) + .16, state.rawPitch ?? state.pitch);
      syncUi();
      if (state.modelSource !== 'walking') scheduleRender();
    }
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
    syncUi(); resizeCanvas();
    await probeWalkingFbx();
    if (state.modelSource === 'walking') {
      startFbxMode();
    } else {
      await loadSuzanneModel();
    }
    animationLoop();
  }

  init();

