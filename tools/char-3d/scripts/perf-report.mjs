import { readFileSync } from 'node:fs';
import { performance } from 'node:perf_hooks';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { parseOBJ } from '../src/mesh/objParser.js';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const objPath = join(root, 'public/models/suzanne.obj');
const source = readFileSync(objPath, 'utf8');
const samples = 12;
const times = [];
let lastMesh = null;

for (let i = 0; i < samples; i++) {
  const t0 = performance.now();
  lastMesh = parseOBJ(source);
  times.push(performance.now() - t0);
}

const avg = times.reduce((sum, value) => sum + value, 0) / times.length;
const min = Math.min(...times);
const max = Math.max(...times);

console.log(`OBJ topology parse benchmark (${samples} samples)`);
console.log(`verts=${lastMesh.verts.length} faces=${lastMesh.faces.length} edges=${lastMesh.edges.length}`);
console.log(`avg=${avg.toFixed(2)}ms min=${min.toFixed(2)}ms max=${max.toFixed(2)}ms`);
