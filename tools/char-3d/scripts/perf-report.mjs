import { readdirSync, readFileSync, statSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { parseOBJ } from '../src/mesh/objParser.js';
import { prepareMeshRuntime } from '../src/mesh/meshRuntime.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const modelDir = join(root, 'public/models');
const objFiles = readdirSync(modelDir).filter(name => name.toLowerCase().endsWith('.obj')).sort();

for (const name of objFiles) {
  const objPath = join(modelDir, name);
  const sizeMb = statSync(objPath).size / (1024 * 1024);
  const source = readFileSync(objPath, 'utf8');
  const samples = sizeMb > 20 ? 2 : sizeMb > 2 ? 4 : 12;
  const times = [];
  let lastMesh = null;

  for (let i = 0; i < samples; i++) {
    const t0 = performance.now();
    lastMesh = parseOBJ(source);
    prepareMeshRuntime(lastMesh);
    times.push(performance.now() - t0);
  }

  const avg = times.reduce((sum, value) => sum + value, 0) / times.length;
  const min = Math.min(...times);
  const max = Math.max(...times);
  console.log(`${name} OBJ parse+runtime benchmark (${samples} samples, ${sizeMb.toFixed(1)} MB)`);
  console.log(`verts=${lastMesh.runtime.vertCount} faces=${lastMesh.runtime.faceCount} edges=${lastMesh.runtime.edgeCount}`);
  console.log(`avg=${avg.toFixed(2)}ms min=${min.toFixed(2)}ms max=${max.toFixed(2)}ms`);
}
