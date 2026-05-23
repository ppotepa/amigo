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
