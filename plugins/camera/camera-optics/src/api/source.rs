use amigo_plugin_api::RenderContributionSet;

use crate::api::CameraOpticalResponse2d;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraOpticalSourceStatus2d {
    Active,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraOpticalEmitterKind2d {
    LightGroup,
    Beacon,
    ParticleLight,
    EmissiveMaterial,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraOpticalSource2d {
    pub owner: String,
    pub component_kind: String,
    pub emitter_kind: CameraOpticalEmitterKind2d,
    pub source_id: Option<String>,
    pub render_layer: Option<String>,
    pub color_rgba: Option<[f32; 4]>,
    pub intensity: Option<f32>,
    pub effective_intensity: Option<f32>,
    pub response: CameraOpticalResponse2d,
    pub status: CameraOpticalSourceStatus2d,
    pub reason: String,
    pub position_px: Option<[f32; 2]>,
    pub radius_px: Option<f32>,
    pub roles: RenderContributionSet,
}
