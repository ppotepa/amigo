import { buildEdges } from './meshEdges.js';
import { prepareMeshRuntime, syncMeshRuntimeVertices } from './meshRuntime.js';

export const FBX_MODEL_URL = '/models/walking.fbx';
const WORLD_UP = [0, 1, 0];
const WORLD_RIGHT = [1, 0, 0];

function v3(x = 0, y = 0, z = 0) { return { x, y, z }; }

function fbxSourceLabel(state, sourceUrl) {
  if (state.fbxSourceLabel) return state.fbxSourceLabel;
  if (sourceUrl === FBX_MODEL_URL) return 'walking.fbx';
  const tail = String(sourceUrl || '').split('/').pop();
  return tail || 'custom.fbx';
}

export function frameObject(object, THREE) {
  const box = new THREE.Box3().setFromObject(object);
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z) || 1;
  object.position.sub(center);
  object.scale.multiplyScalar(2.6 / maxDim);
  object.userData.baseScale = object.scale.x || 1;
  object.rotation.set(0, 0, 0);
}

export function resizeFbxRenderer() {
  return;
}

export function fbxWeldKey(p) {
  const q = 10000;
  return `${Math.round(p.x*q)}_${Math.round(p.y*q)}_${Math.round(p.z*q)}`;
}

function buildFbxMeshAdapter(rt) {
  const THREE = rt.THREE;
  const verts = [];
  const faces = [];
  const entries = [];
  const welded = new Map();
  const tmp = new THREE.Vector3();
  const addWeldedVertex = () => {
    const key = fbxWeldKey(tmp);
    let id = welded.get(key);
    if (id === undefined) {
      id = verts.length;
      welded.set(key, id);
      verts.push(v3(tmp.x, tmp.y, tmp.z));
    }
    return id;
  };
  rt.object.updateMatrixWorld(true);
  rt.object.traverse(child => {
    if (!child.isMesh || !child.geometry?.attributes?.position) return;
    const pos = child.geometry.attributes.position;
    const index = child.geometry.index;
    const localIds = new Int32Array(pos.count);
    for (let i=0;i<pos.count;i++) {
      tmp.fromBufferAttribute(pos, i);
      child.localToWorld(tmp);
      localIds[i] = addWeldedVertex();
    }
    entries.push({ child, pos, index, localIds });
    if (index) {
      for (let i=0;i<index.count;i+=3) {
        const a=localIds[index.getX(i)], b=localIds[index.getX(i+1)], c=localIds[index.getX(i+2)];
        if (a!==b && b!==c && c!==a) faces.push({v:[a,b,c], id:faces.length});
      }
    } else {
      for (let i=0;i<pos.count;i+=3) {
        const a=localIds[i], b=localIds[i+1], c=localIds[i+2];
        if (a!==b && b!==c && c!==a) faces.push({v:[a,b,c], id:faces.length});
      }
    }
  });
  if (!verts.length || !faces.length) return null;
  const mesh = {
    verts,
    faces,
    edges: buildEdges(faces),
    name: rt.label ? `${rt.label} adapter` : 'walking.fbx adapter',
    sourceType: 'fbx',
    cacheId: `fbx:${rt.label || 'walking'}`,
    frameVersion: 0,
  };
  prepareMeshRuntime(mesh);
  return {
    kind: 'mesh',
    entries,
    counts: new Uint16Array(verts.length),
    mesh,
    tmp,
  };
}

function buildSkeletonFallbackAdapter(rt) {
  const THREE = rt.THREE;
  const verts = [];
  const faces = [];
  const segments = [];
  rt.object.updateMatrixWorld(true);
  rt.object.traverse(child => {
    if (!child.isBone || !child.parent?.isBone) return;
    segments.push({ start: child.parent, end: child, base: verts.length });
    for (let i = 0; i < 8; i++) verts.push(v3());
    const b = verts.length - 8;
    faces.push(
      { v: [b + 0, b + 1, b + 5], id: faces.length },
      { v: [b + 0, b + 5, b + 4], id: faces.length + 1 },
      { v: [b + 1, b + 2, b + 6], id: faces.length + 2 },
      { v: [b + 1, b + 6, b + 5], id: faces.length + 3 },
      { v: [b + 2, b + 3, b + 7], id: faces.length + 4 },
      { v: [b + 2, b + 7, b + 6], id: faces.length + 5 },
      { v: [b + 3, b + 0, b + 4], id: faces.length + 6 },
      { v: [b + 3, b + 4, b + 7], id: faces.length + 7 },
      { v: [b + 0, b + 2, b + 1], id: faces.length + 8 },
      { v: [b + 0, b + 3, b + 2], id: faces.length + 9 },
      { v: [b + 4, b + 5, b + 6], id: faces.length + 10 },
      { v: [b + 4, b + 6, b + 7], id: faces.length + 11 },
    );
  });
  if (!segments.length) {
    throw new Error('FBX adapter: file contains no renderable mesh and no bone hierarchy fallback.');
  }
  const mesh = {
    verts,
    faces,
    edges: buildEdges(faces),
    name: rt.label ? `${rt.label} skeleton fallback` : 'walking.fbx skeleton fallback',
    sourceType: 'fbx',
    cacheId: `fbx:${rt.label || 'walking'}:skeleton`,
    frameVersion: 0,
  };
  prepareMeshRuntime(mesh);
  return {
    kind: 'skeleton',
    segments,
    mesh,
    tmp: new THREE.Vector3(),
    tmp2: new THREE.Vector3(),
    tmp3: new THREE.Vector3(),
    tmp4: new THREE.Vector3(),
  };
}

function writeSkeletonSegment(adapter, segment) {
  const verts = adapter.mesh.verts;
  const start = adapter.tmp;
  const end = adapter.tmp2;
  const axis = adapter.tmp3;
  const side = adapter.tmp4;
  segment.start.getWorldPosition(start);
  segment.end.getWorldPosition(end);
  axis.subVectors(end, start);
  const length = axis.length();
  if (length < 1e-5) return;
  axis.multiplyScalar(1 / length);
  side.fromArray(Math.abs(axis.y) < 0.92 ? WORLD_UP : WORLD_RIGHT);
  side.cross(axis);
  if (side.lengthSq() < 1e-8) side.set(1, 0, 0).cross(axis);
  side.normalize();
  const up = axis.clone().cross(side).normalize();
  const startRadius = Math.max(0.015, Math.min(0.09, length * 0.17));
  const endRadius = Math.max(0.012, startRadius * 0.84);
  const sideStart = side.clone().multiplyScalar(startRadius);
  const upStart = up.clone().multiplyScalar(startRadius * 0.72);
  const sideEnd = side.clone().multiplyScalar(endRadius);
  const upEnd = up.clone().multiplyScalar(endRadius * 0.72);
  const base = segment.base;
  const corners = [
    start.clone().add(sideStart).add(upStart),
    start.clone().sub(sideStart).add(upStart),
    start.clone().sub(sideStart).sub(upStart),
    start.clone().add(sideStart).sub(upStart),
    end.clone().add(sideEnd).add(upEnd),
    end.clone().sub(sideEnd).add(upEnd),
    end.clone().sub(sideEnd).sub(upEnd),
    end.clone().add(sideEnd).sub(upEnd),
  ];
  for (let i = 0; i < 8; i++) {
    verts[base + i].x = corners[i].x;
    verts[base + i].y = corners[i].y;
    verts[base + i].z = corners[i].z;
  }
}

function updateFbxAdapterMesh(adapter) {
  if (adapter.kind === 'skeleton') {
    for (const segment of adapter.segments) writeSkeletonSegment(adapter, segment);
    syncMeshRuntimeVertices(adapter.mesh);
    adapter.mesh.frameVersion++;
    return adapter.mesh;
  }
  const verts = adapter.mesh.verts;
  for (const p of verts) {
    p.x = 0;
    p.y = 0;
    p.z = 0;
  }
  adapter.counts.fill(0);
  for (const entry of adapter.entries) {
    const { child, pos, localIds } = entry;
    child.updateMatrixWorld(true);
    for (let i=0;i<pos.count;i++) {
      adapter.tmp.fromBufferAttribute(pos, i);
      if (child.isSkinnedMesh && typeof child.applyBoneTransform === 'function') child.applyBoneTransform(i, adapter.tmp);
      child.localToWorld(adapter.tmp);
      const id = localIds[i];
      const p = verts[id];
      p.x += adapter.tmp.x;
      p.y += adapter.tmp.y;
      p.z += adapter.tmp.z;
      adapter.counts[id]++;
    }
  }
  for (let i=0;i<verts.length;i++) {
    const count = adapter.counts[i] || 1;
    verts[i].x /= count;
    verts[i].y /= count;
    verts[i].z /= count;
  }
  syncMeshRuntimeVertices(adapter.mesh);
  adapter.mesh.frameVersion++;
  return adapter.mesh;
}

export function extractFbxAdapterMesh(rt) {
  if (!rt.fbxAdapter) rt.fbxAdapter = buildFbxMeshAdapter(rt) || buildSkeletonFallbackAdapter(rt);
  rt.object.updateMatrixWorld(true);
  return updateFbxAdapterMesh(rt.fbxAdapter);
}

export async function ensureFbxRuntime(state) {
  const sourceUrl = state.fbxSourceUrl || FBX_MODEL_URL;
  if (state.fbxRuntime && state.fbxRuntime.sourceUrl === sourceUrl) return state.fbxRuntime;
  if (state.fbxRuntime && state.fbxRuntime.sourceUrl !== sourceUrl) {
    state.fbxRuntime = null;
    state.fbxLoadPromise = null;
  }
  if (state.fbxLoadPromise) return state.fbxLoadPromise;
  const loadToken = (state.fbxLoadToken || 0) + 1;
  state.fbxLoadToken = loadToken;
  state.fbxLoadPromise = (async () => {
    const THREE = await import('https://esm.sh/three@0.160.0');
    const { FBXLoader } = await import('https://esm.sh/three@0.160.0/examples/jsm/loaders/FBXLoader.js');
    const scene = new THREE.Scene();
    const loader = new FBXLoader();
    const object = await loader.loadAsync(sourceUrl);
    frameObject(object, THREE);
    scene.add(object);
    const mixer = new THREE.AnimationMixer(object);
    if (object.animations && object.animations.length) mixer.clipAction(object.animations[0]).play();
    const rt = {
      THREE,
      scene,
      object,
      mixer,
      duration: object.animations?.[0]?.duration || 1,
      sourceUrl,
      label: fbxSourceLabel(state, sourceUrl),
    };
    if (state.fbxLoadToken !== loadToken || (state.fbxSourceUrl || FBX_MODEL_URL) !== sourceUrl) {
      return state.fbxRuntime && state.fbxRuntime.sourceUrl === (state.fbxSourceUrl || FBX_MODEL_URL)
        ? state.fbxRuntime
        : rt;
    }
    state.fbxRuntime = rt;
    state.mesh = extractFbxAdapterMesh(rt);
    state.fbxAdapterKind = rt.fbxAdapter?.kind || 'mesh';
    state.frame = null;
    state.fbxError = '';
    state.fbxLoadPromise = null;
    return rt;
  })().catch(err => {
    if (state.fbxLoadToken !== loadToken || (state.fbxSourceUrl || FBX_MODEL_URL) !== sourceUrl) {
      return Promise.reject(err);
    }
    state.fbxError = err && err.message ? err.message : String(err);
    state.fbxRuntime = null;
    state.fbxLoadPromise = null;
    throw err;
  });
  return state.fbxLoadPromise;
}
