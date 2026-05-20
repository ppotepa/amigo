use amigo_plugin_api::{
    render_contributions::roles, CandidateStatus, CandidateTrace, DomainCandidate, DomainId,
    RenderContributionSet, TargetId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraOpticalResponse2dDocument {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub intensity: f32,
    #[serde(default)]
    pub bloom: f32,
    #[serde(default)]
    pub glare: f32,
    #[serde(default)]
    pub ghosting: f32,
    #[serde(default)]
    pub streaks: f32,
    #[serde(default)]
    pub chromatic_smear: f32,
    #[serde(default)]
    pub dirt_response: f32,
    #[serde(default)]
    pub halation: f32,
    #[serde(default)]
    pub threshold: f32,
}

impl Default for CameraOpticalResponse2dDocument {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.0,
            bloom: 0.0,
            glare: 0.0,
            ghosting: 0.0,
            streaks: 0.0,
            chromatic_smear: 0.0,
            dirt_response: 0.0,
            halation: 0.0,
            threshold: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOpticalResponse2dSceneCommand {
    pub enabled: bool,
    pub intensity: f32,
    pub bloom: f32,
    pub glare: f32,
    pub ghosting: f32,
    pub streaks: f32,
    pub chromatic_smear: f32,
    pub dirt_response: f32,
    pub halation: f32,
    pub threshold: f32,
}

pub fn camera_optical_response_from_document(
    response: CameraOpticalResponse2dDocument,
) -> CameraOpticalResponse2dSceneCommand {
    CameraOpticalResponse2dSceneCommand {
        enabled: response.enabled,
        intensity: response.intensity,
        bloom: response.bloom,
        glare: response.glare,
        ghosting: response.ghosting,
        streaks: response.streaks,
        chromatic_smear: response.chromatic_smear,
        dirt_response: response.dirt_response,
        halation: response.halation,
        threshold: response.threshold,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraOpticalResponse2d {
    pub enabled: bool,
    pub intensity: f32,
    pub bloom: f32,
    pub glare: f32,
    pub ghosting: f32,
    pub streaks: f32,
    pub chromatic_smear: f32,
    pub dirt_response: f32,
    pub halation: f32,
    pub threshold: f32,
}

impl Default for CameraOpticalResponse2d {
    fn default() -> Self {
        Self {
            enabled: false,
            intensity: 0.0,
            bloom: 0.0,
            glare: 0.0,
            ghosting: 0.0,
            streaks: 0.0,
            chromatic_smear: 0.0,
            dirt_response: 0.0,
            halation: 0.0,
            threshold: 0.0,
        }
    }
}

impl CameraOpticalResponse2d {
    pub fn normalized(mut self) -> Self {
        self.intensity = finite_or_zero(self.intensity).clamp(0.0, 8.0);
        self.bloom = finite_or_zero(self.bloom).clamp(0.0, 8.0);
        self.glare = finite_or_zero(self.glare).clamp(0.0, 8.0);
        self.ghosting = finite_or_zero(self.ghosting).clamp(0.0, 8.0);
        self.streaks = finite_or_zero(self.streaks).clamp(0.0, 8.0);
        self.chromatic_smear = finite_or_zero(self.chromatic_smear).clamp(0.0, 8.0);
        self.dirt_response = finite_or_zero(self.dirt_response).clamp(0.0, 8.0);
        self.halation = finite_or_zero(self.halation).clamp(0.0, 8.0);
        self.threshold = finite_or_zero(self.threshold).clamp(0.0, 1.0);
        self
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CameraOpticalCoverage2d {
    LightMapChannel { source: String, channel: String },
    Hotspot { entity_name: String, radius_px: f32 },
    Glyphs { entity_name: String, render_layer: String },
    TextureAlpha { entity_name: String, render_layer: String },
    VectorCoverage { entity_name: String, render_layer: String },
    ParticleCoverage { emitter_entity_name: String },
    Unsupported { reason: String },
}

impl CameraOpticalCoverage2d {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::LightMapChannel { .. } => "lightmap_channel",
            Self::Hotspot { .. } => "hotspot",
            Self::Glyphs { .. } => "glyphs",
            Self::TextureAlpha { .. } => "texture_alpha",
            Self::VectorCoverage { .. } => "vector_coverage",
            Self::ParticleCoverage { .. } => "particle_coverage",
            Self::Unsupported { .. } => "unsupported",
        }
    }

    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported { .. })
    }

    pub fn unsupported_reason(&self) -> Option<&str> {
        match self {
            Self::Unsupported { reason } => Some(reason.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraOpticalCandidateStatus2d {
    Active,
    Skipped,
}

impl CameraOpticalCandidateStatus2d {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Skipped => "skipped",
        }
    }

    pub fn as_domain_status(self) -> CandidateStatus {
        match self {
            Self::Active => CandidateStatus::Active,
            Self::Skipped => CandidateStatus::Inactive,
        }
    }
}

pub type CameraOpticalCandidateStatus = CameraOpticalCandidateStatus2d;

#[derive(Debug, Clone, PartialEq)]
pub struct CameraOpticalCandidate2d {
    pub owner: String,
    pub component_kind: String,
    pub render_layer: Option<String>,
    pub color_rgba: [f32; 4],
    pub intensity: f32,
    pub response: CameraOpticalResponse2d,
    pub coverage: CameraOpticalCoverage2d,
    pub roles: RenderContributionSet,
    pub status: CameraOpticalCandidateStatus2d,
    pub reason: String,
    pub position_px: Option<[f32; 2]>,
    pub target_ids: Vec<TargetId>,
    pub trace: Option<CandidateTrace>,
}

impl CameraOpticalCandidate2d {
    pub fn is_active(&self) -> bool {
        self.status == CameraOpticalCandidateStatus2d::Active && self.coverage.is_supported()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.enabled_or(role, false)
    }

    pub fn highlight_gain(&self) -> f32 {
        if !self.is_active() || !self.has_role(roles::CAMERA_FX_SOURCE) {
            return 0.0;
        }

        self.intensity.max(0.0)
            * self
                .response
                .intensity
                .max(self.response.glare)
                .max(self.response.ghosting)
                .max(self.response.streaks)
                .max(self.response.dirt_response)
                .max(self.response.halation)
                .max(0.0)
    }

    pub fn emissive_gain(&self) -> f32 {
        if !self.is_active() || !self.has_role(roles::BLOOM_SOURCE) {
            return 0.0;
        }

        self.intensity.max(0.0) * self.response.intensity.max(self.response.bloom).max(0.0)
    }

    pub fn targets_scene_highlight(&self) -> bool {
        self.highlight_gain() > 0.0
    }

    pub fn targets_scene_emissive(&self) -> bool {
        self.emissive_gain() > 0.0
    }

    pub fn recompute_targets(&mut self) {
        self.target_ids.clear();
        if self.targets_scene_highlight() {
            self.target_ids.push(scene_highlight_target());
        }
        if self.targets_scene_emissive() {
            self.target_ids.push(scene_emissive_target());
        }
    }
}

impl DomainCandidate for CameraOpticalCandidate2d {
    fn domain(&self) -> DomainId {
        DomainId("camera.optics".to_string())
    }

    fn status(&self) -> CandidateStatus {
        self.status.as_domain_status()
    }

    fn target_ids(&self) -> &[TargetId] {
        &self.target_ids
    }

    fn trace(&self) -> Option<&CandidateTrace> {
        self.trace.as_ref()
    }
}

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

pub fn scene_highlight_target() -> TargetId {
    amigo_plugin_api::scene_highlight()
}

pub fn scene_emissive_target() -> TargetId {
    amigo_plugin_api::scene_emissive()
}
