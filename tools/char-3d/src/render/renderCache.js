const PIPELINE_FIELDS = [
  'yaw', 'pitch', 'zoom',
  'cameraYaw', 'cameraPitch', 'cameraX', 'cameraY', 'cameraZ',
  'lightAz', 'lightEl',
  'method', 'mode', 'flowMode',
  'density', 'layers', 'threshold', 'core', 'contact', 'edgeDark', 'simplify', 'economy',
  'strokeLen', 'spacing', 'strokeWidth', 'curvature', 'crossAngle', 'dotSize',
  'wobble', 'jitter', 'strokeCrookedness', 'strokeKinkChance', 'strokeToneRamp',
  'shadowFrameDrift', 'shadowLoopRedraw', 'shadowLayoutJitter',
  'projectionWobble', 'spacingVar', 'lengthVar', 'widthVar', 'taper', 'breakup', 'overdraw',
  'contourHumanize', 'contourDrift', 'contourWobble', 'contourGaps', 'contourFrameVariance',
  'shadowsEnabled', 'hideOccluded', 'backface', 'depthClipStrokes', 'clipToFaces',
  'showHidden', 'depthEps', 'creases', 'suggestive', 'contactLines',
  'animFrameIndex', 'animLoopIndex', 'animSampleTime', 'animJitterFrames',
];

const BACKGROUND_FIELDS = [
  'paintPaperColor',
];

const PAINT_FIELDS = [
  'paintEnabled', 'paintBrush', 'faceWash', 'tone', 'sortFaces', 'backface',
  'paintBaseColor', 'paintShadowColor', 'paintHighlightColor', 'paintPaperColor',
  'paintBaseOpacity', 'paintWashOpacity', 'paintCelStrength', 'paintCelSteps',
  'paintHighlightAmount', 'paintHalftone', 'paintHalftoneScale', 'paintGrain',
  'paintRegistration', 'paintBleed', 'paintRegionResolution', 'paintRegionSimplify',
  'paintEdgeBleed', 'paintPigmentGranulation', 'paintRegionJitter', 'paintWetMix',
];

function createLayerRecord() {
  return {
    key: '',
    canvas: null,
    ctx: null,
    w: 0,
    h: 0,
  };
}

export function createRenderCache() {
  return {
    frame: null,
    pipelineKey: '',
    background: createLayerRecord(),
    paint: createLayerRecord(),
    svg: {
      key: '',
      text: '',
    },
    depth: {
      w: 0,
      h: 0,
      depth: null,
      owner: null,
    },
  };
}

export function buildPipelineKey(state, canvas) {
  const values = PIPELINE_FIELDS.map(field => state[field]);
  const mesh = state.mesh;
  return JSON.stringify({
    size: [canvas.width, canvas.height],
    mesh: mesh ? [mesh.cacheId || mesh.name || 'mesh', mesh.sourceType || 'unknown', mesh.frameVersion || 0] : null,
    values,
  });
}

export function buildBackgroundLayerKey(state, canvas) {
  return JSON.stringify({
    size: [canvas.width, canvas.height],
    values: BACKGROUND_FIELDS.map(field => state[field]),
  });
}

export function buildPaintLayerKey(state, canvas, frame) {
  return JSON.stringify({
    size: [canvas.width, canvas.height],
    frame: frame ? [frame.faces.length, frame.contours?.length || 0, frame.marks?.length || 0] : null,
    values: PAINT_FIELDS.map(field => state[field]),
  });
}

export function buildSvgKey(state, frame) {
  return JSON.stringify({
    frame: frame ? [frame.faces.length, frame.contours?.length || 0, frame.marks?.length || 0] : null,
    mesh: state.mesh ? [state.mesh.cacheId || state.mesh.name || 'mesh', state.mesh.frameVersion || 0] : null,
    values: [
      state.paintEnabled, state.paintBaseColor, state.paintShadowColor, state.paintHighlightColor,
      state.paintBaseOpacity, state.paintCelStrength, state.paintCelSteps, state.paintHighlightAmount,
      state.paintBrush, state.paintRegionResolution, state.paintRegionSimplify, state.paintEdgeBleed,
      state.paintPigmentGranulation, state.paintRegionJitter, state.paintWetMix,
      state.contours, state.inkDominance, state.hideOccluded, state.method, state.flowMode, state.mode,
      state.backface, state.sortFaces, state.yaw, state.pitch, state.cameraYaw, state.cameraPitch,
    ],
  });
}

export function ensureLayerCanvas(layer, w, h) {
  if (layer.canvas && layer.w === w && layer.h === h && layer.ctx) return layer;
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  layer.canvas = canvas;
  layer.ctx = canvas.getContext('2d', { alpha: true });
  layer.w = w;
  layer.h = h;
  layer.key = '';
  return layer;
}

export function invalidateDerivedCaches(cache) {
  cache.background.key = '';
  cache.paint.key = '';
  cache.svg.key = '';
  cache.svg.text = '';
}

export function getReusableDepthBuffers(cache, w, h) {
  if (!cache.depth.depth || cache.depth.w !== w || cache.depth.h !== h) {
    cache.depth.w = w;
    cache.depth.h = h;
    cache.depth.depth = new Float32Array(w * h);
    cache.depth.owner = new Int32Array(w * h);
  }
  cache.depth.depth.fill(-1e9);
  cache.depth.owner.fill(-1);
  return cache.depth;
}
