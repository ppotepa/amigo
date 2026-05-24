export function buildRenderSelection(detailResult, runtime, state) {
  const items = (detailResult.items || detailResult || [])
    .filter(item => item.detailTier < 4)
    .sort((a, b) => {
      if (a.detailTier !== b.detailTier) return a.detailTier - b.detailTier;
      return b.projectedAreaPx - a.projectedAreaPx;
    });

  const faceSet = new Set();
  const edgeSet = new Set();
  const vertexSet = new Set();
  const faceTier = new Int8Array(runtime.faceCount);
  const edgeTier = new Int8Array(runtime.edgeCount);
  const faceUnit = new Int32Array(runtime.faceCount);
  faceTier.fill(4);
  edgeTier.fill(4);
  faceUnit.fill(-1);

  const maxFaces = state.vectorBudgetEnabled ? Math.max(0, Number(state.vectorMaxProjectedFaces) || 0) : Infinity;
  const maxEdges = state.vectorBudgetEnabled ? Math.max(0, Number(state.vectorMaxVisibleEdges) || 0) : Infinity;
  let budgetFacesHit = false;
  let budgetEdgesHit = false;

  for (const item of items) {
    const unit = item.unit;
    const tier = item.detailTier;

    for (let i = 0; i < unit.faceIds.length; i++) {
      if (faceSet.size >= maxFaces) {
        budgetFacesHit = true;
        break;
      }
      const faceId = unit.faceIds[i];
      if (faceSet.has(faceId)) continue;
      faceSet.add(faceId);
      faceTier[faceId] = tier;
      faceUnit[faceId] = unit.id;
      vertexSet.add(runtime.faceA[faceId]);
      vertexSet.add(runtime.faceB[faceId]);
      vertexSet.add(runtime.faceC[faceId]);
    }

    for (let i = 0; i < unit.edgeIds.length; i++) {
      if (edgeSet.size >= maxEdges) {
        budgetEdgesHit = true;
        break;
      }
      const edgeId = unit.edgeIds[i];
      if (edgeSet.has(edgeId)) continue;
      edgeSet.add(edgeId);
      edgeTier[edgeId] = Math.min(edgeTier[edgeId], tier);
      vertexSet.add(runtime.edgeA[edgeId]);
      vertexSet.add(runtime.edgeB[edgeId]);
    }
  }

  const localToGlobalVertex = Int32Array.from(vertexSet);
  const globalToLocalVertex = new Int32Array(runtime.vertCount);
  globalToLocalVertex.fill(-1);
  for (let i = 0; i < localToGlobalVertex.length; i++) globalToLocalVertex[localToGlobalVertex[i]] = i;

  return {
    units: items,
    vertexIds: localToGlobalVertex,
    localToGlobalVertex,
    globalToLocalVertex,
    faceIds: Int32Array.from(faceSet),
    edgeIds: Int32Array.from(edgeSet),
    faceTier,
    edgeTier,
    faceUnit,
    counters: {
      selectedUnits: items.length,
      selectedVertices: vertexSet.size,
      selectedFaces: faceSet.size,
      selectedEdges: edgeSet.size,
      budgetFacesHit: budgetFacesHit ? 1 : 0,
      budgetEdgesHit: budgetEdgesHit ? 1 : 0,
    },
  };
}

export function buildFullRenderSelection(runtime) {
  const vertexIds = new Int32Array(runtime.vertCount);
  const faceIds = new Int32Array(runtime.faceCount);
  const edgeIds = new Int32Array(runtime.edgeCount);
  for (let i = 0; i < runtime.vertCount; i++) vertexIds[i] = i;
  for (let i = 0; i < runtime.faceCount; i++) faceIds[i] = i;
  for (let i = 0; i < runtime.edgeCount; i++) edgeIds[i] = i;
  const faceTier = new Int8Array(runtime.faceCount);
  const edgeTier = new Int8Array(runtime.edgeCount);
  const faceUnit = new Int32Array(runtime.faceCount);
  const globalToLocalVertex = new Int32Array(runtime.vertCount);
  faceUnit.fill(0);
  for (let i = 0; i < runtime.vertCount; i++) globalToLocalVertex[i] = i;
  return {
    units: [],
    vertexIds,
    localToGlobalVertex: vertexIds,
    globalToLocalVertex,
    faceIds,
    edgeIds,
    faceTier,
    edgeTier,
    faceUnit,
    counters: {
      selectedUnits: 1,
      selectedVertices: runtime.vertCount,
      selectedFaces: runtime.faceCount,
      selectedEdges: runtime.edgeCount,
      budgetFacesHit: 0,
      budgetEdgesHit: 0,
    },
  };
}
