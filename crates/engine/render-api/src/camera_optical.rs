use crate::RenderContributionSet;

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
}

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
}

impl CameraOpticalCandidate2d {
    pub fn is_active(&self) -> bool {
        self.status == CameraOpticalCandidateStatus2d::Active && self.coverage.is_supported()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.enabled_or(role, false)
    }

    pub fn highlight_gain(&self) -> f32 {
        if !self.is_active() || !self.has_role(crate::render_contribution_roles::CAMERA_FX_SOURCE)
        {
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
        if !self.is_active() || !self.has_role(crate::render_contribution_roles::BLOOM_SOURCE) {
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
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CameraOpticalCandidate2d, CameraOpticalCandidateStatus2d, CameraOpticalCoverage2d,
        CameraOpticalResponse2d,
    };
    use crate::{render_contribution_roles as roles, RenderContributionSet};

    #[test]
    fn camera_optical_response_normalizes_values() {
        let response = CameraOpticalResponse2d {
            enabled: true,
            intensity: 12.0,
            bloom: f32::NAN,
            threshold: 2.0,
            ..CameraOpticalResponse2d::default()
        }
        .normalized();

        assert_eq!(response.intensity, 8.0);
        assert_eq!(response.bloom, 0.0);
        assert_eq!(response.threshold, 1.0);
    }

    #[test]
    fn camera_optical_coverage_reports_stable_kind() {
        assert_eq!(
            CameraOpticalCoverage2d::LightMapChannel {
                source: "map".to_owned(),
                channel: "neon".to_owned(),
            }
            .kind(),
            "lightmap_channel"
        );
    }

    fn candidate_with_roles(roles: &[&str]) -> CameraOpticalCandidate2d {
        let mut role_set = RenderContributionSet::default();
        for role in roles {
            role_set.set(*role, true);
        }
        CameraOpticalCandidate2d {
            owner: "neon".to_owned(),
            component_kind: "LightGroup2D".to_owned(),
            render_layer: None,
            color_rgba: [1.0, 1.0, 1.0, 1.0],
            intensity: 2.0,
            response: CameraOpticalResponse2d {
                enabled: true,
                intensity: 0.25,
                bloom: 0.5,
                glare: 0.75,
                ..CameraOpticalResponse2d::default()
            },
            coverage: CameraOpticalCoverage2d::Hotspot {
                entity_name: "neon".to_owned(),
                radius_px: 16.0,
            },
            roles: role_set,
            status: CameraOpticalCandidateStatus2d::Active,
            reason: "camera_optical_candidate_active".to_owned(),
            position_px: Some([10.0, 20.0]),
        }
    }

    #[test]
    fn camera_optical_candidate_bloom_role_does_not_target_highlight() {
        let candidate = candidate_with_roles(&[roles::BLOOM_SOURCE]);

        assert_eq!(candidate.highlight_gain(), 0.0);
        assert!(!candidate.targets_scene_highlight());
    }

    #[test]
    fn camera_optical_candidate_camera_fx_role_targets_highlight() {
        let candidate = candidate_with_roles(&[roles::CAMERA_FX_SOURCE]);

        assert_eq!(candidate.highlight_gain(), 1.5);
        assert!(candidate.targets_scene_highlight());
        assert!(!candidate.targets_scene_emissive());
    }

    #[test]
    fn camera_optical_candidate_bloom_role_targets_emissive() {
        let candidate = candidate_with_roles(&[roles::BLOOM_SOURCE]);

        assert_eq!(candidate.emissive_gain(), 1.0);
        assert!(candidate.targets_scene_emissive());
    }

    #[test]
    fn camera_optical_candidate_unsupported_coverage_has_zero_gains() {
        let mut candidate = candidate_with_roles(&[roles::CAMERA_FX_SOURCE, roles::BLOOM_SOURCE]);
        candidate.coverage = CameraOpticalCoverage2d::Unsupported {
            reason: "unsupported_for_test".to_owned(),
        };

        assert!(!candidate.is_active());
        assert_eq!(candidate.coverage.unsupported_reason(), Some("unsupported_for_test"));
        assert_eq!(candidate.highlight_gain(), 0.0);
        assert_eq!(candidate.emissive_gain(), 0.0);
    }
}
