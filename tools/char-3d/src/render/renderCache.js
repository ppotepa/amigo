const PIPELINE_FIELDS = [
  'controlMode', 'angleSnap', 'yaw', 'pitch', 'zoom',
  'cameraYaw', 'cameraPitch', 'cameraX', 'cameraY', 'cameraZ',
  'projectionMode', 'focalLength',
  'lightAz', 'lightEl',
  'method', 'mode', 'flowMode',
  'density', 'layers', 'threshold', 'core', 'contact', 'edgeDark', 'simplify', 'economy',
  'cleanupMinFaceAreaPx', 'cleanupMinLineLengthPx', 'cleanupMaxEdgeLengthPx', 'cleanupDensityClamp',
  'cleanupRegionMinAreaPx', 'cleanupRegionMinFaces', 'cleanupRegionMaxAspect', 'hairRegionSuppression',
  'scenePartitionEnabled', 'scenePartitionMode', 'scenePartitionCellSize', 'scenePartitionMaxUnits',
  'visibilityCullingEnabled', 'visibilityMarginPx', 'visibilityMinAreaPx', 'visibilityMinRadiusPx',
  'detailPolicyEnabled', 'detailTier0RadiusPx', 'detailTier1RadiusPx', 'detailTier2RadiusPx', 'detailTier3RadiusPx',
  'detailDensityPenalty', 'detailImportanceBias',
  'vectorBudgetEnabled', 'vectorMaxProjectedFaces', 'vectorMaxVisibleEdges', 'vectorMaxContourLines',
  'vectorMaxShadowMarks', 'vectorMinFaceAreaPx', 'vectorMinEdgeLengthPx',
  'regionBudgetEnabled', 'regionMinProjectedAreaPx', 'regionMaxPaintRegions', 'regionAllowFarFills',
  'shadowBandCount', 'shadowRegionBleed', 'shadowColorJitter',
  'strokePressureJitter', 'temporalCoherence', 'projectionHumanError',
  'strokeLen', 'spacing', 'strokeWidth', 'curvature', 'crossAngle', 'dotSize',
  'wobble', 'jitter', 'strokeCrookedness', 'strokeKinkChance', 'strokeToneRamp',
  'shadowFrameDrift', 'shadowLoopRedraw', 'shadowLayoutJitter',
  'projectionWobble', 'spacingVar', 'lengthVar', 'widthVar', 'taper', 'breakup', 'overdraw',
  'contourHumanize', 'contourDrift', 'contourWobble', 'contourGaps', 'contourFrameVariance',
  'shadowsEnabled', 'paintEnabled', 'contours', 'flow',
  'mainContourEnabled', 'creaseAccentEnabled', 'suggestiveContourEnabled', 'hiddenLineEnabled', 'shadowHatchEnabled',
  'mainContourTool', 'creaseAccentTool', 'suggestiveContourTool', 'hiddenLineTool', 'shadowHatchTool',
  'hideOccluded', 'backface', 'depthClipStrokes', 'clipToFaces',
  'showHidden', 'depthEps', 'creases', 'suggestive', 'contactLines',
  'animFrameIndex', 'animLoopIndex', 'animSampleTime', 'animJitterFrames',
];

const BACKGROUND_FIELDS = [
  'paintPaperColor',
];

const PAINT_FIELDS = [
  'paintEnabled', 'paintBrush', 'faceWash', 'tone', 'sortFaces', 'backface',
  'baseWashEnabled', 'shadowRegionEnabled', 'highlightRegionEnabled',
  'paintBaseColor', 'paintShadowColor', 'paintHighlightColor', 'paintPaperColor',
  'paintBaseOpacity', 'paintWashOpacity', 'paintCelStrength', 'paintCelSteps',
  'paintHighlightAmount', 'paintHalftone', 'paintHalftoneScale', 'paintGrain',
  'paintRegistration', 'paintBleed', 'paintRegionResolution', 'paintRegionSimplify',
  'paintEdgeBleed', 'paintPigmentGranulation', 'paintRegionJitter', 'paintWetMix',
  'cleanupRegionMinAreaPx', 'cleanupRegionMinFaces', 'cleanupRegionMaxAspect',
  'regionBudgetEnabled', 'regionMinProjectedAreaPx', 'regionMaxPaintRegions', 'regionAllowFarFills',
  'detailPolicyEnabled', 'detailTier2RadiusPx', 'detailTier3RadiusPx',
  'hairRegionSuppression', 'shadowBandCount', 'shadowRegionBleed', 'shadowColorJitter',
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

function createFrameLists() {
  return {
    verts: [],
    faces: [],
    screenFaces: [],
    visibleFaces: [],
    sortedFaces: [],
    contours: [],
    renderSelection: null,
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
    partition: {
      key: '',
      value: null,
    },
    depth: {
      w: 0,
      h: 0,
      depth: null,
      owner: null,
    },
    frameLists: createFrameLists(),
  };
}

export function clearFrameLists(cache) {
  const lists = cache.frameLists || (cache.frameLists = createFrameLists());
  lists.screenFaces.length = 0;
  lists.visibleFaces.length = 0;
  lists.sortedFaces.length = 0;
  lists.contours.length = 0;
  lists.renderSelection = null;
  return lists;
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
    projection: frame?.pipelineKey || null,
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
      state.cleanupRegionMinAreaPx, state.cleanupRegionMinFaces, state.cleanupRegionMaxAspect,
      state.hairRegionSuppression, state.shadowBandCount, state.shadowRegionBleed, state.shadowColorJitter,
      state.contours, state.inkDominance, state.hideOccluded, state.method, state.flowMode, state.mode,
      state.backface, state.sortFaces, state.controlMode, state.angleSnap, state.yaw, state.pitch, state.zoom, state.cameraYaw, state.cameraPitch,
      state.cameraX, state.cameraY, state.cameraZ, state.projectionMode, state.focalLength,
      state.strokeTools, state.lineSets, state.regionSets,
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
