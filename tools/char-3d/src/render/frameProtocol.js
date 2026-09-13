export const FRAME_WORKER_FACE_THRESHOLD = 100000;

export function meshFrameWorkerKey(mesh) {
  if (!mesh) return '';
  return `${mesh.cacheId || mesh.name || 'mesh'}:${mesh.sourceType || 'unknown'}:${mesh.frameVersion || 0}`;
}

export function shouldUseFrameWorker(mesh, runtime) {
  return Boolean(
    typeof Worker !== 'undefined' &&
    mesh?.sourceType === 'obj' &&
    runtime?.faceCount >= FRAME_WORKER_FACE_THRESHOLD
  );
}

export function buildFrameWorkerMeshPayload(mesh, runtime) {
  return {
    name: mesh?.name || 'mesh',
    sourceType: mesh?.sourceType || 'unknown',
    vertCount: runtime.vertCount,
    faceCount: runtime.faceCount,
    edgeCount: runtime.edgeCount,
    vertX: runtime.vertX,
    vertY: runtime.vertY,
    vertZ: runtime.vertZ,
    faceA: runtime.faceA,
    faceB: runtime.faceB,
    faceC: runtime.faceC,
    edgeA: runtime.edgeA,
    edgeB: runtime.edgeB,
    edgeF0: runtime.edgeF0,
    edgeF1: runtime.edgeF1,
  };
}

export function snapshotFrameWorkerParams(state, canvas, helpers) {
  return {
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
    lightAz: state.lightAz,
    lightEl: state.lightEl,
    projectionWobble: state.projectionWobble,
    randomSeed: helpers.randomSeed,
    cameraDollyScale: helpers.cameraDollyScale,
    cullMargin: helpers.cullMargin,
    edgeDark: state.edgeDark,
    contact: state.contact,
    core: state.core,
    simplify: state.simplify,
    cleanupMinFaceAreaPx: state.cleanupMinFaceAreaPx,
    cleanupMinLineLengthPx: state.cleanupMinLineLengthPx,
    cleanupMaxEdgeLengthPx: state.cleanupMaxEdgeLengthPx,
    scenePartitionEnabled: state.scenePartitionEnabled,
    scenePartitionMode: state.scenePartitionMode,
    scenePartitionCellSize: state.scenePartitionCellSize,
    scenePartitionMaxUnits: state.scenePartitionMaxUnits,
    visibilityCullingEnabled: state.visibilityCullingEnabled,
    visibilityMarginPx: state.visibilityMarginPx,
    visibilityMinAreaPx: state.visibilityMinAreaPx,
    visibilityMinRadiusPx: state.visibilityMinRadiusPx,
    detailPolicyEnabled: state.detailPolicyEnabled,
    detailTier0RadiusPx: state.detailTier0RadiusPx,
    detailTier1RadiusPx: state.detailTier1RadiusPx,
    detailTier2RadiusPx: state.detailTier2RadiusPx,
    detailTier3RadiusPx: state.detailTier3RadiusPx,
    detailDensityPenalty: state.detailDensityPenalty,
    detailImportanceBias: state.detailImportanceBias,
    vectorBudgetEnabled: state.vectorBudgetEnabled,
    vectorMaxProjectedFaces: state.vectorMaxProjectedFaces,
    vectorMaxVisibleEdges: state.vectorMaxVisibleEdges,
    vectorMaxContourLines: state.vectorMaxContourLines,
    vectorMaxShadowMarks: state.vectorMaxShadowMarks,
    vectorMinFaceAreaPx: state.vectorMinFaceAreaPx,
    vectorMinEdgeLengthPx: state.vectorMinEdgeLengthPx,
    backface: state.backface,
    hideOccluded: state.hideOccluded,
    depthClipStrokes: state.depthClipStrokes,
    showHidden: state.showHidden,
    depthEps: state.depthEps,
    creases: state.creases,
    suggestive: state.suggestive,
    contours: state.contours,
    shadowsEnabled: state.shadowsEnabled,
    flow: state.flow,
    paintEnabled: state.paintEnabled,
    faceWash: state.faceWash,
    tone: state.tone,
    flowMode: state.flowMode,
  };
}
