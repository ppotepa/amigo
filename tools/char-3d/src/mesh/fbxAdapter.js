import { buildEdges } from './meshEdges.js';

export const FBX_MODEL_URL = '/models/walking.fbx';

function v3(x = 0, y = 0, z = 0) { return { x, y, z }; }

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

function buildFbxAdapter(rt) {
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
  if (!verts.length || !faces.length) throw new Error('FBX adapter: no renderable mesh geometry found.');
  return {
    entries,
    counts: new Uint16Array(verts.length),
    mesh: {
      verts,
      faces,
      edges: buildEdges(faces),
      name: 'walking.fbx adapter',
      sourceType: 'fbx',
      cacheId: 'fbx:walking',
      frameVersion: 0,
    },
    tmp,
  };
}

function updateFbxAdapterMesh(adapter) {
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
  adapter.mesh.frameVersion++;
  return adapter.mesh;
}

export function extractFbxAdapterMesh(rt) {
  if (!rt.fbxAdapter) rt.fbxAdapter = buildFbxAdapter(rt);
  rt.object.updateMatrixWorld(true);
  return updateFbxAdapterMesh(rt.fbxAdapter);
}

export async function ensureFbxRuntime(state) {
  if (state.fbxRuntime) return state.fbxRuntime;
  if (state.fbxLoadPromise) return state.fbxLoadPromise;
  state.fbxLoadPromise = (async () => {
    const THREE = await import('https://esm.sh/three@0.160.0');
    const { FBXLoader } = await import('https://esm.sh/three@0.160.0/examples/jsm/loaders/FBXLoader.js');
    const scene = new THREE.Scene();
    const loader = new FBXLoader();
    const object = await loader.loadAsync(FBX_MODEL_URL);
    frameObject(object, THREE);
    scene.add(object);
    const mixer = new THREE.AnimationMixer(object);
    if (object.animations && object.animations.length) mixer.clipAction(object.animations[0]).play();
    const rt = { THREE, scene, object, mixer, duration: object.animations?.[0]?.duration || 1 };
    state.fbxRuntime = rt;
    state.mesh = extractFbxAdapterMesh(rt);
    state.frame = null;
    state.fbxError = '';
    return rt;
  })().catch(err => {
    state.fbxError = err && err.message ? err.message : String(err);
    state.fbxRuntime = null;
    state.fbxLoadPromise = null;
    throw err;
  });
  return state.fbxLoadPromise;
}
