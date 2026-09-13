import { boundsDiagonal, createEmptyBounds, expandBoundsByTriangle, finalizeBounds } from './bounds.js';

function partitionKey(runtime, state) {
  return JSON.stringify({
    mesh: [runtime.vertCount, runtime.faceCount, runtime.edgeCount],
    source: state.mesh ? [state.mesh.cacheId || state.mesh.name || 'mesh', state.mesh.frameVersion || 0] : null,
    mode: state.scenePartitionMode || 'spatial',
    cellSize: Math.max(1, Number(state.scenePartitionCellSize) || 24),
    maxUnits: Math.max(1, Number(state.scenePartitionMaxUnits) || 4096),
  });
}

export function getScenePartitionKey(runtime, state) {
  return partitionKey(runtime, state);
}

export function buildScenePartition(runtime, state) {
  if (!runtime) return { key: '', units: [], faceToUnit: new Int32Array(0), edgeToUnit: new Int32Array(0) };
  if (!state.scenePartitionEnabled) return buildSingleUnit(runtime, state);
  return buildSpatialPartition(runtime, state);
}

function buildSingleUnit(runtime, state) {
  const bounds = createEmptyBounds();
  const faceIds = [];
  const vertexSet = new Set();
  for (let id = 0; id < runtime.faceCount; id++) {
    const a = runtime.faceA[id];
    const b = runtime.faceB[id];
    const c = runtime.faceC[id];
    faceIds.push(id);
    vertexSet.add(a);
    vertexSet.add(b);
    vertexSet.add(c);
    expandBoundsByTriangle(
      bounds,
      runtime.vertX[a], runtime.vertY[a], runtime.vertZ[a],
      runtime.vertX[b], runtime.vertY[b], runtime.vertZ[b],
      runtime.vertX[c], runtime.vertY[c], runtime.vertZ[c],
    );
  }
  finalizeBounds(bounds);
  const edgeIds = Array.from({ length: runtime.edgeCount }, (_, i) => i);
  return {
    key: partitionKey(runtime, state),
    units: [{
      id: 0,
      kind: 'single',
      key: 'single',
      faceIds: Int32Array.from(faceIds),
      edgeIds: Int32Array.from(edgeIds),
      vertexIds: Int32Array.from([...vertexSet]),
      bounds,
      importance: 1,
      densityEstimate: densityForUnit(faceIds.length, bounds),
    }],
    faceToUnit: new Int32Array(runtime.faceCount).fill(0),
    edgeToUnit: new Int32Array(runtime.edgeCount).fill(0),
  };
}

function buildSpatialPartition(runtime, state) {
  const cellSize = Math.max(1, Number(state.scenePartitionCellSize) || 24);
  const maxUnits = Math.max(1, Number(state.scenePartitionMaxUnits) || 4096);
  const map = new Map();
  const faceToUnit = new Int32Array(runtime.faceCount);
  faceToUnit.fill(-1);
  let cellScale = 1;

  function keyForCentroid(cx, cy, cz) {
    return `${Math.floor(cx / (cellSize * cellScale))}:${Math.floor(cy / (cellSize * cellScale))}:${Math.floor(cz / (cellSize * cellScale))}`;
  }

  function getUnit(key) {
    let unit = map.get(key);
    if (!unit) {
      unit = {
        id: map.size,
        kind: 'spatial',
        key,
        faceIds: [],
        edgeIds: [],
        vertexSet: new Set(),
        bounds: createEmptyBounds(),
        importance: 1,
        densityEstimate: 0,
      };
      map.set(key, unit);
    }
    return unit;
  }

  for (let id = 0; id < runtime.faceCount; id++) {
    const a = runtime.faceA[id];
    const b = runtime.faceB[id];
    const c = runtime.faceC[id];
    const cx = (runtime.vertX[a] + runtime.vertX[b] + runtime.vertX[c]) / 3;
    const cy = (runtime.vertY[a] + runtime.vertY[b] + runtime.vertY[c]) / 3;
    const cz = (runtime.vertZ[a] + runtime.vertZ[b] + runtime.vertZ[c]) / 3;
    if (map.size > maxUnits) cellScale *= 2;
    const unit = getUnit(keyForCentroid(cx, cy, cz));
    faceToUnit[id] = unit.id;
    unit.faceIds.push(id);
    unit.vertexSet.add(a);
    unit.vertexSet.add(b);
    unit.vertexSet.add(c);
    expandBoundsByTriangle(
      unit.bounds,
      runtime.vertX[a], runtime.vertY[a], runtime.vertZ[a],
      runtime.vertX[b], runtime.vertY[b], runtime.vertZ[b],
      runtime.vertX[c], runtime.vertY[c], runtime.vertZ[c],
    );
  }

  const units = [...map.values()];
  for (const unit of units) {
    finalizeBounds(unit.bounds);
    unit.densityEstimate = densityForUnit(unit.faceIds.length, unit.bounds);
  }

  const edgeToUnit = new Int32Array(runtime.edgeCount);
  edgeToUnit.fill(-1);
  for (let edgeId = 0; edgeId < runtime.edgeCount; edgeId++) {
    const f0 = runtime.edgeF0[edgeId];
    const f1 = runtime.edgeF1[edgeId];
    const u0 = f0 >= 0 ? faceToUnit[f0] : -1;
    const u1 = f1 >= 0 ? faceToUnit[f1] : -1;
    const unitId = u0 >= 0 ? u0 : u1;
    if (unitId < 0) continue;
    edgeToUnit[edgeId] = unitId;
    units[unitId].edgeIds.push(edgeId);
    if (u1 >= 0 && u1 !== unitId) units[u1].edgeIds.push(edgeId);
  }

  for (const unit of units) {
    unit.faceIds = Int32Array.from(unit.faceIds);
    unit.edgeIds = Int32Array.from([...new Set(unit.edgeIds)]);
    unit.vertexIds = Int32Array.from([...unit.vertexSet]);
    delete unit.vertexSet;
  }

  return { key: partitionKey(runtime, state), units, faceToUnit, edgeToUnit };
}

function densityForUnit(faceCount, bounds) {
  const d = Math.max(0.001, boundsDiagonal(bounds));
  return faceCount / (d * d);
}
