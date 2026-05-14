use crate::{CameraBinding, RenderLayerId, RenderSpace, RenderTargetPlan};

#[derive(Debug, Clone, PartialEq)]
pub struct CompositionLayer {
    pub id: RenderLayerId,
    pub space: RenderSpace,
    pub camera: Option<CameraBinding>,
    pub order: i32,
    pub target: RenderTargetPlan,
    pub clear: ClearMode,
    pub depth: DepthMode,
    pub blend: BlendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearMode {
    Inherit,
    ClearColor,
    Preserve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthMode {
    None,
    ReadWrite,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Opaque,
    Alpha,
    Additive,
}
