use crate::{
    ComicInk, FeatureClass, FeatureSegment, NprDebugView, NprFillTriangle, NprGeometry,
    NprCamera, TessellatedStroke, TopologyEdge,
    NprTemporalState,
};
use glam::Vec2;

/// Stable, inspectable stages of every `nprpipeline` composition. Profiles
/// swap strategies inside these stages; they never create a parallel renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NprPipelineStage {
    Surface,
    Features,
    Salience,
    Value,
    Marks,
    Hatching,
    StrokeChains,
    Gesture,
    Paper,
    Medium,
}

pub const DEFAULT_NPR_PIPELINE_STAGES: &[NprPipelineStage] = &[
    NprPipelineStage::Surface,
    NprPipelineStage::Features,
    NprPipelineStage::Salience,
    NprPipelineStage::Value,
    NprPipelineStage::Marks,
    NprPipelineStage::Hatching,
    NprPipelineStage::StrokeChains,
    NprPipelineStage::Gesture,
    NprPipelineStage::Paper,
    NprPipelineStage::Medium,
];

/// Immutable input supplied by a scene/domain extractor to an NPR pipeline.
#[derive(Debug, Clone, Copy)]
pub struct NprPipelineInput<'a> {
    pub geometry: &'a NprGeometry,
    pub camera: NprCamera,
    pub viewport: [u32; 2],
    pub style: ComicInk,
    pub seed: u64,
    pub debug_view: NprDebugView,
    pub temporal: NprTemporalState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprLogicalMark {
    pub id: u32,
    pub class: FeatureClass,
    /// A projected paper-space path. Feature and hatching strategies may
    /// start with two points; a stroke-chain strategy may turn neighbouring
    /// segments into a single human gesture before tessellation.
    pub points: Vec<(Vec2, f32)>,
}

/// Data shared between pipeline stages. It deliberately contains no WGPU types.
pub struct NprPipelineContext<'a> {
    pub input: NprPipelineInput<'a>,
    pub topology: Vec<TopologyEdge>,
    pub features: Vec<FeatureSegment>,
    pub selected_features: Vec<FeatureSegment>,
    pub fills: Vec<NprFillTriangle>,
    pub marks: Vec<NprLogicalMark>,
    pub strokes: Vec<TessellatedStroke>,
}

impl<'a> NprPipelineContext<'a> {
    pub fn new(input: NprPipelineInput<'a>) -> Self {
        Self {
            input,
            topology: Vec::new(),
            features: Vec::new(),
            selected_features: Vec::new(),
            fills: Vec::new(),
            marks: Vec::new(),
            strokes: Vec::new(),
        }
    }
}
