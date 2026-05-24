import fs from 'node:fs/promises';
import path from 'node:path';
import * as THREE from 'three';
import { FBXLoader } from 'three/examples/jsm/loaders/FBXLoader.js';
import { buildFbxClipAmc } from '../src/mesh/fbxClipBake.js';
import { extractFbxAdapterMesh, frameObject } from '../src/mesh/fbxAdapter.js';

const root = process.cwd();
const input = process.argv[2] || path.join(root, 'public/models/walking.fbx');
const output = process.argv[3] || path.join(root, 'rust-impl/assets/models/walking.amc');
const fps = Number(process.argv[4] || 60);

const bytes = await fs.readFile(input);
const loader = new FBXLoader();
const object = loader.parse(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength), path.dirname(input));
frameObject(object, THREE);

const mixer = new THREE.AnimationMixer(object);
if (object.animations?.length) mixer.clipAction(object.animations[0]).play();

const rt = {
  THREE,
  object,
  mixer,
  duration: object.animations?.[0]?.duration || 1,
  label: path.basename(input),
};

const buffer = buildFbxClipAmc(rt, { fps, duration: rt.duration });
await fs.mkdir(path.dirname(output), { recursive: true });
await fs.writeFile(output, Buffer.from(buffer));

const mesh = extractFbxAdapterMesh(rt);
console.log(`Wrote ${output}`);
console.log(`fps=${fps} duration=${rt.duration.toFixed(3)}s verts=${mesh.verts.length} faces=${mesh.faces.length}`);
