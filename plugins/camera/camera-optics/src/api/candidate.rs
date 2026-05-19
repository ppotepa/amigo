use amigo_plugin_api::{
    render_contributions::roles, CandidateStatus, CandidateTrace, DomainCandidate, DomainId,
    RenderContributionSet, TargetId,
};

use super::coverage::CameraOpticalCoverage2d;
use super::response::CameraOpticalResponse2d;
use super::targets::{scene_emissive_target, scene_highlight_target};

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
