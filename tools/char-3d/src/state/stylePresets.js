const cleanInkLineSets = {
  mainContour: { enabled: true, source: ['silhouette', 'boundary'], tool: 'mainInk', minLengthPx: 4, simplifyPx: 0.7, priority: 100 },
  creaseAccent: { enabled: true, source: ['crease'], tool: 'thinPen', minLengthPx: 8, priority: 80 },
  suggestiveContour: { enabled: true, source: ['suggestive'], tool: 'softPencil', strength: 0.45, minLengthPx: 8, priority: 60 },
  hiddenLine: { enabled: true, source: ['hidden'], tool: 'hiddenGuide', minLengthPx: 6, priority: 20 },
  shadowHatch: { enabled: true, source: ['toneBand', 'shadow'], tool: 'shadowPencil', method: 'hatching', densityByTone: true, priority: 40 }
};

const pencilLineSets = {
  ...cleanInkLineSets,
  mainContour: { ...cleanInkLineSets.mainContour, tool: 'softPencil', minLengthPx: 3 },
  creaseAccent: { ...cleanInkLineSets.creaseAccent, tool: 'thinPen', minLengthPx: 5 },
  shadowHatch: { ...cleanInkLineSets.shadowHatch, tool: 'shadowPencil' }
};

const comicLineSets = {
  ...cleanInkLineSets,
  mainContour: { ...cleanInkLineSets.mainContour, tool: 'mainInk', minLengthPx: 5 },
  creaseAccent: { ...cleanInkLineSets.creaseAccent, tool: 'mainInk', minLengthPx: 9 },
  suggestiveContour: { ...cleanInkLineSets.suggestiveContour, enabled: false },
  shadowHatch: { ...cleanInkLineSets.shadowHatch, tool: 'thinPen' }
};

export const presets = {
    cleanInk: { mode:'INK', method:'hatching', flowMode:'mixed', density:.72, layers:1, threshold:.20, core:1.0, spacing:13, strokeLen:44, strokeWidth:1.05, curvature:.18, wobble:.20, jitter:.12, breakup:.02, taper:.62, economy:.34, dotSize:2.1, widthVar:.12, spacingVar:.18, lengthVar:.18 },
    cleanInkTight: { mode:'INK', method:'hatching', flowMode:'parallel', density:.92, layers:1, threshold:.18, core:1.08, spacing:10, strokeLen:42, strokeWidth:.98, curvature:.12, wobble:.14, jitter:.08, breakup:.01, taper:.58, economy:.26, widthVar:.10, spacingVar:.14, lengthVar:.15 },
    cleanInkLoose: { mode:'INK', method:'hatching', flowMode:'mixed', density:.66, layers:1, threshold:.22, core:.95, spacing:15, strokeLen:52, strokeWidth:1.12, curvature:.28, wobble:.30, jitter:.18, breakup:.03, taper:.68, economy:.40, widthVar:.14, spacingVar:.24, lengthVar:.26 },
    contourInk: { mode:'INK', method:'contourHatch', flowMode:'crossContour', density:.78, layers:1, threshold:.18, core:1.1, spacing:12, strokeLen:54, strokeWidth:1.05, curvature:.55, wobble:.24, jitter:.15, breakup:.02, taper:.65, economy:.28, spacingVar:.18, lengthVar:.22 },
    contourWrap: { mode:'INK', method:'contourHatch', flowMode:'crossContour', density:.90, layers:2, threshold:.14, core:1.18, spacing:11, strokeLen:58, strokeWidth:.92, curvature:.76, wobble:.26, jitter:.16, breakup:.03, taper:.70, economy:.22, crossAngle:48, spacingVar:.20, lengthVar:.28 },
    engraving: { mode:'INK', method:'crosshatch', flowMode:'parallel', density:1.08, layers:3, threshold:.14, core:1.35, spacing:9, strokeLen:60, strokeWidth:.72, curvature:.08, wobble:.06, jitter:.04, breakup:.0, taper:.28, economy:.18, crossAngle:52, widthVar:.06, spacingVar:.06, lengthVar:.08 },
    crossClassic: { mode:'INK', method:'crosshatch', flowMode:'mixed', density:.96, layers:2, threshold:.16, core:1.25, spacing:10, strokeLen:50, strokeWidth:.86, curvature:.12, wobble:.10, jitter:.08, breakup:.01, taper:.36, economy:.20, crossAngle:58, widthVar:.08, spacingVar:.12, lengthVar:.14 },
    crossLoose: { mode:'INK', method:'crosshatch', flowMode:'mixed', density:.82, layers:2, threshold:.18, core:1.12, spacing:12, strokeLen:54, strokeWidth:.90, curvature:.26, wobble:.24, jitter:.18, breakup:.03, taper:.48, economy:.28, crossAngle:44, widthVar:.12, spacingVar:.20, lengthVar:.24 },
    architectPen: { mode:'INK', method:'hatching', flowMode:'silhouette', density:.62, layers:1, threshold:.22, core:1.08, spacing:14, strokeLen:36, strokeWidth:.92, curvature:.10, wobble:.08, jitter:.06, breakup:.00, taper:.26, economy:.44, edgeDark:.42, contact:.54, widthVar:.04, spacingVar:.08, lengthVar:.10 },
    loosePencil: { mode:'PENCIL', method:'hatching', flowMode:'mixed', density:.95, layers:2, threshold:.13, core:.85, spacing:10, strokeLen:38, strokeWidth:.72, curvature:.36, wobble:.56, jitter:.45, breakup:.08, taper:.44, economy:.18, overdraw:.28, widthVar:.38, spacingVar:.30, lengthVar:.34 },
    softSketch: { mode:'PENCIL', method:'hatching', flowMode:'mixed', density:.84, layers:2, threshold:.16, core:.82, spacing:12, strokeLen:44, strokeWidth:.68, curvature:.48, wobble:.62, jitter:.48, breakup:.10, taper:.48, economy:.24, overdraw:.34, widthVar:.42, spacingVar:.34, lengthVar:.38 },
    graphite: { mode:'PENCIL', method:'graphite', flowMode:'mixed', density:1.15, layers:3, threshold:.10, core:.8, spacing:8, strokeLen:32, strokeWidth:.55, curvature:.40, wobble:.42, jitter:.36, breakup:.04, taper:.30, economy:.10, overdraw:.40, widthVar:.42, spacingVar:.26, lengthVar:.28 },
    graphiteDark: { mode:'PENCIL', method:'graphite', flowMode:'mixed', density:1.35, layers:4, threshold:.08, core:.92, spacing:7, strokeLen:28, strokeWidth:.62, curvature:.46, wobble:.46, jitter:.38, breakup:.04, taper:.28, economy:.08, overdraw:.52, widthVar:.46, spacingVar:.22, lengthVar:.22 },
    stipple: { mode:'INK', method:'stipple', flowMode:'mixed', density:1.25, layers:1, threshold:.11, core:1.2, spacing:9, strokeLen:25, strokeWidth:.8, dotSize:2.0, jitter:.62, economy:.12, spacingVar:.24 },
    stippleFine: { mode:'INK', method:'stipple', flowMode:'mixed', density:1.48, layers:1, threshold:.09, core:1.32, spacing:7, strokeLen:20, strokeWidth:.65, dotSize:1.4, jitter:.42, economy:.08, spacingVar:.18 },
    halftone: { mode:'INK', method:'halftone', flowMode:'parallel', density:.95, layers:1, threshold:.08, core:1.45, spacing:12, dotSize:3.2, jitter:.09, economy:.08 },
    halftoneBold: { mode:'INK', method:'halftone', flowMode:'parallel', density:1.12, layers:1, threshold:.06, core:1.62, spacing:10, dotSize:4.2, jitter:.06, economy:.05 },
    dryBrush: { mode:'BRUSH', method:'drybrush', flowMode:'mixed', density:.86, layers:1, threshold:.12, core:1.15, spacing:14, strokeLen:66, strokeWidth:2.25, curvature:.35, wobble:.62, jitter:.32, breakup:.58, taper:.72, economy:.23, widthVar:.55, spacingVar:.28, lengthVar:.34 },
    brushWash: { mode:'BRUSH', method:'drybrush', flowMode:'light', density:.72, layers:1, threshold:.15, core:.96, spacing:16, strokeLen:74, strokeWidth:1.85, curvature:.28, wobble:.48, jitter:.22, breakup:.24, taper:.62, economy:.30, widthVar:.38, spacingVar:.24, lengthVar:.30 },
    concept: { mode:'PENCIL', method:'scribble', flowMode:'mixed', density:.95, layers:2, threshold:.10, core:.9, spacing:13, strokeLen:42, strokeWidth:.85, curvature:.72, wobble:.75, jitter:.55, breakup:.08, taper:.22, economy:.18, overdraw:.45, widthVar:.34, spacingVar:.34, lengthVar:.44 },
    conceptLoose: { mode:'PENCIL', method:'scribble', flowMode:'mixed', density:.72, layers:2, threshold:.14, core:.82, spacing:16, strokeLen:54, strokeWidth:.96, curvature:.85, wobble:.82, jitter:.62, breakup:.10, taper:.18, economy:.26, overdraw:.55, widthVar:.36, spacingVar:.42, lengthVar:.54 },
    manga: { mode:'INK', method:'comic', flowMode:'terminator', density:.92, layers:2, threshold:.25, core:1.9, spacing:9, strokeLen:52, strokeWidth:1.35, curvature:.16, wobble:.16, jitter:.08, breakup:.0, taper:.78, economy:.30, edgeDark:.55, contact:.7, widthVar:.14 },
    comicHeavy: { mode:'INK', method:'comic', flowMode:'terminator', density:1.08, layers:2, threshold:.22, core:2.10, spacing:8, strokeLen:58, strokeWidth:1.65, curvature:.12, wobble:.12, jitter:.06, breakup:.0, taper:.84, economy:.22, edgeDark:.68, contact:.82, widthVar:.10 },
    hybrid: { mode:'BRUSH', method:'hybrid', flowMode:'mixed', density:.95, layers:2, threshold:.13, core:1.05, spacing:11, strokeLen:48, strokeWidth:1.20, dotSize:1.8, curvature:.38, wobble:.42, jitter:.30, breakup:.25, taper:.55, economy:.18, spacingVar:.24, lengthVar:.24 },
    hybridSoft: { mode:'PENCIL', method:'hybrid', flowMode:'mixed', density:.86, layers:2, threshold:.15, core:.92, spacing:12, strokeLen:42, strokeWidth:.92, dotSize:1.5, curvature:.42, wobble:.52, jitter:.36, breakup:.18, taper:.50, economy:.22, spacingVar:.26, lengthVar:.30 },
    featherLight: { mode:'INK', method:'feather', flowMode:'terminator', density:.76, layers:1, threshold:.18, core:.92, spacing:14, strokeLen:36, strokeWidth:.92, curvature:.22, wobble:.18, jitter:.10, breakup:.02, taper:.88, economy:.36, edgeDark:.24, contact:.36 },
    featherDeep: { mode:'INK', method:'feather', flowMode:'terminator', density:1.02, layers:2, threshold:.14, core:1.26, spacing:10, strokeLen:44, strokeWidth:1.08, curvature:.26, wobble:.20, jitter:.12, breakup:.02, taper:.92, economy:.24, edgeDark:.34, contact:.46 },
    scumbleCharcoal: { mode:'BRUSH', method:'scumble', flowMode:'mixed', density:.96, layers:2, threshold:.12, core:.92, spacing:12, strokeLen:26, strokeWidth:1.20, curvature:.78, wobble:.72, jitter:.58, breakup:.18, taper:.20, economy:.16, overdraw:.46, widthVar:.44, spacingVar:.36, lengthVar:.40 },
    scumbleDust: { mode:'PENCIL', method:'scumble', flowMode:'mixed', density:.74, layers:2, threshold:.16, core:.80, spacing:14, strokeLen:24, strokeWidth:.88, curvature:.82, wobble:.68, jitter:.52, breakup:.14, taper:.18, economy:.24, overdraw:.34, widthVar:.36, spacingVar:.38, lengthVar:.42 },
    techEtch: { mode:'INK', method:'crosshatch', flowMode:'parallel', density:1.18, layers:4, threshold:.12, core:1.42, spacing:8, strokeLen:56, strokeWidth:.68, curvature:.05, wobble:.04, jitter:.03, breakup:0, taper:.22, economy:.14, crossAngle:60, edgeDark:.36, contact:.42, widthVar:.04, spacingVar:.04, lengthVar:.06 },
    pipelineCleanInk: { mode:'INK', method:'hatching', flowMode:'mixed', density:.76, layers:1, threshold:.20, core:1.05, spacing:13, strokeLen:44, strokeWidth:1.0, cleanupMinFaceAreaPx:2, cleanupMinLineLengthPx:4, cleanupDensityClamp:.65, cleanupRegionMinAreaPx:120, cleanupRegionMinFaces:4, cleanupRegionMaxAspect:14, shadowBandCount:3, temporalCoherence:.9, projectionHumanError:.08, strokePressureJitter:.16, lineSets: cleanInkLineSets },
    pipelinePencilStudy: { mode:'PENCIL', method:'graphite', flowMode:'mixed', density:1.1, layers:3, threshold:.12, core:.82, spacing:9, strokeLen:34, strokeWidth:.65, cleanupMinFaceAreaPx:1.5, cleanupMinLineLengthPx:2.5, cleanupDensityClamp:.72, shadowBandCount:4, temporalCoherence:.86, projectionHumanError:.16, strokePressureJitter:.34, lineSets: pencilLineSets },
    pipelineBrushWash: { mode:'BRUSH', method:'drybrush', flowMode:'light', density:.72, layers:1, threshold:.15, core:.96, spacing:16, strokeLen:74, strokeWidth:1.85, cleanupRegionMinAreaPx:180, cleanupRegionMinFaces:5, shadowBandCount:3, shadowRegionBleed:.36, shadowColorJitter:.34, temporalCoherence:.82, projectionHumanError:.14, lineSets: { ...cleanInkLineSets, shadowHatch: { ...cleanInkLineSets.shadowHatch, tool: 'mainInk' } } },
    pipelineComicShadow: { mode:'INK', method:'comic', flowMode:'terminator', density:1.04, layers:2, threshold:.23, core:2.0, spacing:8, strokeLen:56, strokeWidth:1.55, cleanupDensityClamp:.58, cleanupRegionMinAreaPx:160, cleanupRegionMinFaces:5, shadowBandCount:2, shadowColorJitter:.12, temporalCoherence:.94, projectionHumanError:.05, lineSets: comicLineSets },
    pipelineDenseHairSafe: { mode:'PENCIL', method:'hybrid', flowMode:'mixed', density:.86, layers:2, threshold:.15, spacing:12, strokeLen:40, strokeWidth:.88, cleanupMinFaceAreaPx:4, cleanupMinLineLengthPx:5, cleanupDensityClamp:.48, cleanupRegionMinAreaPx:260, cleanupRegionMinFaces:8, cleanupRegionMaxAspect:9, hairRegionSuppression:.92, shadowBandCount:3, temporalCoherence:.9, lineSets: { ...pencilLineSets, suggestiveContour: { ...pencilLineSets.suggestiveContour, enabled: false } } },
    pipelineNoisyHandDrawn: { mode:'PENCIL', method:'scribble', flowMode:'mixed', density:.82, layers:2, threshold:.13, spacing:15, strokeLen:48, strokeWidth:.9, wobble:.8, jitter:.62, cleanupDensityClamp:.72, contourDrift:2.3, contourWobble:.36, contourGaps:.14, strokePressureJitter:.55, projectionHumanError:.32, temporalCoherence:.58, lineSets: pencilLineSets },
    largeSceneBalanced: {
      scenePartitionEnabled: true, scenePartitionMode: 'spatial', scenePartitionCellSize: 32, visibilityCullingEnabled: true,
      visibilityMarginPx: 80, visibilityMinAreaPx: 3, visibilityMinRadiusPx: 1.8, detailPolicyEnabled: true,
      detailTier0RadiusPx: 180, detailTier1RadiusPx: 80, detailTier2RadiusPx: 28, detailTier3RadiusPx: 8,
      detailDensityPenalty: .35, vectorBudgetEnabled: true, vectorMaxProjectedFaces: 12000, vectorMaxVisibleEdges: 8000,
      vectorMaxContourLines: 5000, vectorMaxShadowMarks: 900, vectorMinFaceAreaPx: 1.2, vectorMinEdgeLengthPx: 2.2,
      regionBudgetEnabled: true, regionMinProjectedAreaPx: 48, regionMaxPaintRegions: 260, regionAllowFarFills: false
    },
    closeSubjectDetail: {
      scenePartitionEnabled: true, scenePartitionMode: 'spatial', scenePartitionCellSize: 18, visibilityCullingEnabled: true,
      visibilityMarginPx: 90, visibilityMinAreaPx: .5, visibilityMinRadiusPx: .8, detailPolicyEnabled: true,
      detailTier0RadiusPx: 120, detailTier1RadiusPx: 54, detailTier2RadiusPx: 18, detailTier3RadiusPx: 5,
      detailDensityPenalty: .12, vectorBudgetEnabled: true, vectorMaxProjectedFaces: 36000, vectorMaxVisibleEdges: 26000,
      vectorMaxContourLines: 14000, vectorMaxShadowMarks: 4200, vectorMinFaceAreaPx: .35, vectorMinEdgeLengthPx: .8,
      regionBudgetEnabled: true, regionMinProjectedAreaPx: 16, regionMaxPaintRegions: 850, regionAllowFarFills: true
    },
    denseSceneCleanup: {
      scenePartitionEnabled: true, scenePartitionMode: 'spatial', scenePartitionCellSize: 20, visibilityCullingEnabled: true,
      visibilityMarginPx: 60, visibilityMinAreaPx: 6, visibilityMinRadiusPx: 2.6, detailPolicyEnabled: true,
      detailTier0RadiusPx: 220, detailTier1RadiusPx: 110, detailTier2RadiusPx: 42, detailTier3RadiusPx: 12,
      detailDensityPenalty: .85, vectorBudgetEnabled: true, vectorMaxProjectedFaces: 9000, vectorMaxVisibleEdges: 6000,
      vectorMaxContourLines: 3500, vectorMaxShadowMarks: 650, vectorMinFaceAreaPx: 2.4, vectorMinEdgeLengthPx: 3.2,
      regionBudgetEnabled: true, regionMinProjectedAreaPx: 96, regionMaxPaintRegions: 160, regionAllowFarFills: false
    }
  };

